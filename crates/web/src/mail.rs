use std::{env, fmt};

use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    transport::smtp::authentication::Credentials,
};

/// How the SMTP connection is secured. `Plain` is the local Mailpit path;
/// a relay on the network needs `StartTls`, which also carries a login.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SmtpSecurity {
    Plain,
    StartTls { username: String, password: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub security: SmtpSecurity,
    pub from: String,
    pub base_url: String,
}

impl MailConfig {
    /// Reads the mail configuration. `SMTP_TLS` unset or `off` keeps the
    /// plaintext Mailpit path; `starttls` requires `SMTP_USERNAME` and
    /// `SMTP_PASSWORD`. Any other value is refused so a typo cannot silently
    /// send credentials in the clear.
    pub fn try_from_env() -> Result<Self, MailError> {
        Self::from_lookup(|name| env::var(name).ok())
    }

    /// The same reading over any source of variables, so the rules can be
    /// tested without touching the process environment.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self, MailError> {
        let read = |name: &str| lookup(name).filter(|value| !value.trim().is_empty());
        let security = match read("SMTP_TLS").as_deref() {
            None | Some("off") => SmtpSecurity::Plain,
            Some("starttls") => SmtpSecurity::StartTls {
                username: read("SMTP_USERNAME")
                    .ok_or_else(|| MailError("SMTP_TLS=starttls requires SMTP_USERNAME".into()))?,
                password: read("SMTP_PASSWORD")
                    .ok_or_else(|| MailError("SMTP_TLS=starttls requires SMTP_PASSWORD".into()))?,
            },
            Some(_) => return Err(MailError("SMTP_TLS must be off or starttls".into())),
        };
        let default_port = match security {
            SmtpSecurity::Plain => 1025,
            SmtpSecurity::StartTls { .. } => 587,
        };
        let smtp_port = match read("SMTP_PORT") {
            None => default_port,
            Some(value) => value
                .trim()
                .parse()
                .map_err(|_| MailError("SMTP_PORT must be a port number".into()))?,
        };
        Ok(Self {
            smtp_host: read("SMTP_HOST").unwrap_or_else(|| "127.0.0.1".into()),
            smtp_port,
            security,
            from: read("MAIL_FROM").unwrap_or_else(|| "DAC2 <no-reply@dac2.local>".into()),
            base_url: read("APP_BASE_URL").unwrap_or_else(|| "http://127.0.0.1:3000".into()),
        })
    }
}

#[derive(Debug)]
pub struct MailError(String);

impl fmt::Display for MailError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for MailError {}

#[derive(Clone)]
pub struct Mailer {
    config: MailConfig,
    transport: AsyncSmtpTransport<Tokio1Executor>,
}

impl Mailer {
    pub fn new(config: MailConfig) -> Result<Self, MailError> {
        let transport = match &config.security {
            SmtpSecurity::Plain => {
                AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(config.smtp_host.clone())
                    .port(config.smtp_port)
                    .build()
            }
            SmtpSecurity::StartTls { username, password } => {
                AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_host)
                    .map_err(|error| MailError(format!("SMTP relay is not usable: {error}")))?
                    .port(config.smtp_port)
                    .credentials(Credentials::new(username.clone(), password.clone()))
                    .build()
            }
        };
        Ok(Self { config, transport })
    }

    pub fn from_env() -> Result<Self, MailError> {
        MailConfig::try_from_env().and_then(Self::new)
    }

    pub async fn send_verification(&self, recipient: &str, token: &str) -> Result<(), MailError> {
        self.send(
            recipient,
            "ยืนยันอีเมล DAC2",
            "auth/verify-email",
            token,
            "กดลิงก์นี้เพื่อยืนยันอีเมล",
        )
        .await
    }

    pub async fn send_password_reset(&self, recipient: &str, token: &str) -> Result<(), MailError> {
        self.send(
            recipient,
            "ตั้งรหัสผ่าน DAC2 ใหม่",
            "reset-password",
            token,
            "กดลิงก์นี้เพื่อตั้งรหัสผ่านใหม่",
        )
        .await
    }

    async fn send(
        &self,
        recipient: &str,
        subject: &str,
        path: &str,
        token: &str,
        instruction: &str,
    ) -> Result<(), MailError> {
        let link = self.link(path, token);
        let message = Message::builder()
            .from(
                self.config
                    .from
                    .parse()
                    .map_err(|error| MailError(format!("invalid sender: {error}")))?,
            )
            .to(recipient
                .parse()
                .map_err(|error| MailError(format!("invalid recipient: {error}")))?)
            .subject(subject)
            .body(format!("{instruction}\n\n{link}\n"))
            .map_err(|error| MailError(format!("could not build message: {error}")))?;

        self.transport
            .send(message)
            .await
            .map_err(|error| MailError(format!("SMTP delivery failed: {error}")))?;
        Ok(())
    }

    fn link(&self, path: &str, token: &str) -> String {
        format!(
            "{}/{path}?token={token}",
            self.config.base_url.trim_end_matches('/')
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_use_the_configured_local_base_without_logging() {
        let mailer = Mailer::new(MailConfig {
            smtp_host: "127.0.0.1".into(),
            smtp_port: 1025,
            security: SmtpSecurity::Plain,
            from: "DAC2 <no-reply@dac2.local>".into(),
            base_url: "http://127.0.0.1:3000/".into(),
        })
        .expect("a plaintext transport needs no relay lookup");
        assert_eq!(
            mailer.link("auth/verify-email", "secret-token"),
            "http://127.0.0.1:3000/auth/verify-email?token=secret-token"
        );
    }

    fn lookup(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let pairs: Vec<(String, String)> = pairs
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .collect();
        move |name| {
            pairs
                .iter()
                .find(|(candidate, _)| candidate == name)
                .map(|(_, value)| value.clone())
        }
    }

    #[test]
    fn unset_mail_variables_keep_the_local_plaintext_path() {
        let config = MailConfig::from_lookup(lookup(&[])).expect("defaults are valid");
        assert_eq!(config.smtp_host, "127.0.0.1");
        assert_eq!(config.smtp_port, 1025);
        assert_eq!(config.security, SmtpSecurity::Plain);
        assert_eq!(config.base_url, "http://127.0.0.1:3000");
    }

    #[test]
    fn starttls_carries_the_login_and_defaults_to_submission_port() {
        let config = MailConfig::from_lookup(lookup(&[
            ("SMTP_TLS", "starttls"),
            ("SMTP_HOST", "smtp-relay.example"),
            ("SMTP_USERNAME", "login"),
            ("SMTP_PASSWORD", "secret"),
        ]))
        .expect("a complete STARTTLS configuration is valid");
        assert_eq!(config.smtp_port, 587);
        assert_eq!(
            config.security,
            SmtpSecurity::StartTls {
                username: "login".into(),
                password: "secret".into(),
            }
        );
    }

    #[test]
    fn starttls_without_a_login_is_refused_by_name() {
        let error = MailConfig::from_lookup(lookup(&[
            ("SMTP_TLS", "starttls"),
            ("SMTP_USERNAME", "login"),
        ]))
        .expect_err("a missing password is a configuration error");
        assert_eq!(
            error.to_string(),
            "SMTP_TLS=starttls requires SMTP_PASSWORD"
        );
    }

    #[test]
    fn an_unknown_tls_mode_is_refused_rather_than_sent_in_the_clear() {
        let error = MailConfig::from_lookup(lookup(&[("SMTP_TLS", "tls")]))
            .expect_err("only off and starttls are modes");
        assert_eq!(error.to_string(), "SMTP_TLS must be off or starttls");
    }

    #[test]
    fn a_port_that_is_not_a_number_is_refused() {
        let error = MailConfig::from_lookup(lookup(&[("SMTP_PORT", "five")]))
            .expect_err("a port must parse");
        assert_eq!(error.to_string(), "SMTP_PORT must be a port number");
    }
}
