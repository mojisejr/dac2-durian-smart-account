use std::{env, fmt};

use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::Mailbox,
    transport::smtp::authentication::Credentials,
};

/// How the SMTP connection is secured. `Plain` is the local Mailpit path;
/// a relay on the network needs `StartTls`, which also carries a login.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SmtpSecurity {
    Plain,
    StartTls { username: String, password: String },
}

/// How a message leaves the process. SMTP is the local path and the path for
/// a host that allows outbound SMTP; a host that blocks the SMTP ports, as a
/// free Render web service does, reaches the relay over HTTPS instead.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Delivery {
    Smtp {
        host: String,
        port: u16,
        security: SmtpSecurity,
    },
    BrevoApi {
        api_key: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailConfig {
    pub delivery: Delivery,
    pub from: String,
    pub base_url: String,
}

/// Where the Brevo transactional API lives. A test overrides it.
pub const BREVO_API_URL: &str = "https://api.brevo.com/v3/smtp/email";

impl MailConfig {
    /// Reads the mail configuration. `MAIL_TRANSPORT` unset or `smtp` reads
    /// the SMTP variables: `SMTP_TLS` unset or `off` keeps the plaintext
    /// Mailpit path, `starttls` requires `SMTP_USERNAME` and `SMTP_PASSWORD`.
    /// `MAIL_TRANSPORT=brevo-api` requires `BREVO_API_KEY` and ignores the
    /// SMTP variables. Any other word is refused so a typo cannot silently
    /// send credentials in the clear or send nothing at all.
    pub fn try_from_env() -> Result<Self, MailError> {
        Self::from_lookup(|name| env::var(name).ok())
    }

    /// The same reading over any source of variables, so the rules can be
    /// tested without touching the process environment.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self, MailError> {
        let read = |name: &str| lookup(name).filter(|value| !value.trim().is_empty());
        let delivery = match read("MAIL_TRANSPORT").as_deref() {
            None | Some("smtp") => {
                let security = match read("SMTP_TLS").as_deref() {
                    None | Some("off") => SmtpSecurity::Plain,
                    Some("starttls") => SmtpSecurity::StartTls {
                        username: read("SMTP_USERNAME").ok_or_else(|| {
                            MailError("SMTP_TLS=starttls requires SMTP_USERNAME".into())
                        })?,
                        password: read("SMTP_PASSWORD").ok_or_else(|| {
                            MailError("SMTP_TLS=starttls requires SMTP_PASSWORD".into())
                        })?,
                    },
                    Some(_) => return Err(MailError("SMTP_TLS must be off or starttls".into())),
                };
                let default_port = match security {
                    SmtpSecurity::Plain => 1025,
                    SmtpSecurity::StartTls { .. } => 587,
                };
                let port = match read("SMTP_PORT") {
                    None => default_port,
                    Some(value) => value
                        .trim()
                        .parse()
                        .map_err(|_| MailError("SMTP_PORT must be a port number".into()))?,
                };
                Delivery::Smtp {
                    host: read("SMTP_HOST").unwrap_or_else(|| "127.0.0.1".into()),
                    port,
                    security,
                }
            }
            Some("brevo-api") => Delivery::BrevoApi {
                api_key: read("BREVO_API_KEY").ok_or_else(|| {
                    MailError("MAIL_TRANSPORT=brevo-api requires BREVO_API_KEY".into())
                })?,
            },
            Some(_) => return Err(MailError("MAIL_TRANSPORT must be smtp or brevo-api".into())),
        };
        Ok(Self {
            delivery,
            from: read("MAIL_FROM").unwrap_or_else(|| "DAC2 <no-reply@dac2.local>".into()),
            base_url: read("APP_BASE_URL").unwrap_or_else(|| "http://127.0.0.1:3000".into()),
        })
    }

    /// One line for the start-up log: where mail goes and how, never a login
    /// or a key.
    pub fn describe(&self) -> String {
        match &self.delivery {
            Delivery::Smtp {
                host,
                port,
                security: SmtpSecurity::Plain,
            } => format!("mail relay {host}:{port} (plaintext, local only)"),
            Delivery::Smtp {
                host,
                port,
                security: SmtpSecurity::StartTls { .. },
            } => format!("mail relay {host}:{port} (STARTTLS with login)"),
            Delivery::BrevoApi { .. } => "mail via the Brevo HTTP API (key configured)".into(),
        }
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
enum Transport {
    Smtp(AsyncSmtpTransport<Tokio1Executor>),
    BrevoApi {
        client: reqwest::Client,
        api_key: String,
        url: String,
    },
}

#[derive(Clone)]
pub struct Mailer {
    config: MailConfig,
    transport: Transport,
}

impl Mailer {
    pub fn new(config: MailConfig) -> Result<Self, MailError> {
        Self::with_brevo_url(config, BREVO_API_URL)
    }

    /// `new` with the Brevo endpoint chosen by the caller, so a test can point
    /// the mailer at a local server.
    pub fn with_brevo_url(config: MailConfig, brevo_url: &str) -> Result<Self, MailError> {
        let transport = match &config.delivery {
            Delivery::Smtp {
                host,
                port,
                security: SmtpSecurity::Plain,
            } => Transport::Smtp(
                AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(host.clone())
                    .port(*port)
                    .build(),
            ),
            Delivery::Smtp {
                host,
                port,
                security: SmtpSecurity::StartTls { username, password },
            } => Transport::Smtp(
                AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(host)
                    .map_err(|error| MailError(format!("SMTP relay is not usable: {error}")))?
                    .port(*port)
                    .credentials(Credentials::new(username.clone(), password.clone()))
                    .build(),
            ),
            Delivery::BrevoApi { api_key } => Transport::BrevoApi {
                client: reqwest::Client::builder()
                    // ring is the one TLS provider in this binary (sqlx and
                    // lettre already use it); reqwest is told so explicitly
                    // rather than pulling in a second one.
                    .use_preconfigured_tls(
                        rustls::ClientConfig::builder_with_provider(std::sync::Arc::new(
                            rustls::crypto::ring::default_provider(),
                        ))
                        .with_safe_default_protocol_versions()
                        .map_err(|error| MailError(format!("TLS is not usable: {error}")))?
                        .with_root_certificates(rustls::RootCertStore {
                            roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
                        })
                        .with_no_client_auth(),
                    )
                    .timeout(std::time::Duration::from_secs(20))
                    .build()
                    .map_err(|error| MailError(format!("HTTP client is not usable: {error}")))?,
                api_key: api_key.clone(),
                url: brevo_url.to_owned(),
            },
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
        let body = format!("{instruction}\n\n{link}\n");
        let sender: Mailbox = self
            .config
            .from
            .parse()
            .map_err(|error| MailError(format!("invalid sender: {error}")))?;
        let recipient: Mailbox = recipient
            .parse()
            .map_err(|error| MailError(format!("invalid recipient: {error}")))?;

        match &self.transport {
            Transport::Smtp(transport) => {
                let message = Message::builder()
                    .from(sender)
                    .to(recipient)
                    .subject(subject)
                    .body(body)
                    .map_err(|error| MailError(format!("could not build message: {error}")))?;
                transport
                    .send(message)
                    .await
                    .map_err(|error| MailError(format!("SMTP delivery failed: {error}")))?;
            }
            Transport::BrevoApi {
                client,
                api_key,
                url,
            } => {
                let response = client
                    .post(url)
                    .header("api-key", api_key)
                    .header(reqwest::header::ACCEPT, "application/json")
                    .json(&brevo_request(&sender, &recipient, subject, &body))
                    .send()
                    .await
                    .map_err(|error| {
                        // reqwest's Display names the URL, never the key.
                        MailError(format!("Brevo API request failed: {error}"))
                    })?;
                // The body is not read on failure: Brevo echoes the request,
                // recipient included, and that does not belong in a log.
                if !response.status().is_success() {
                    return Err(MailError(format!(
                        "Brevo API answered {}",
                        response.status()
                    )));
                }
            }
        }
        Ok(())
    }

    fn link(&self, path: &str, token: &str) -> String {
        format!(
            "{}/{path}?token={token}",
            self.config.base_url.trim_end_matches('/')
        )
    }
}

/// The Brevo transactional request, kept as a pure function so its shape is
/// tested without a network.
fn brevo_request(
    sender: &Mailbox,
    recipient: &Mailbox,
    subject: &str,
    text: &str,
) -> serde_json::Value {
    let mut from = serde_json::json!({ "email": sender.email.to_string() });
    if let Some(name) = &sender.name {
        from["name"] = serde_json::Value::String(name.clone());
    }
    serde_json::json!({
        "sender": from,
        "to": [{ "email": recipient.email.to_string() }],
        "subject": subject,
        "textContent": text,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn local_smtp() -> Delivery {
        Delivery::Smtp {
            host: "127.0.0.1".into(),
            port: 1025,
            security: SmtpSecurity::Plain,
        }
    }

    #[test]
    fn links_use_the_configured_local_base_without_logging() {
        let mailer = Mailer::new(MailConfig {
            delivery: local_smtp(),
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
        assert_eq!(config.delivery, local_smtp());
        assert_eq!(config.base_url, "http://127.0.0.1:3000");
        assert_eq!(
            config.describe(),
            "mail relay 127.0.0.1:1025 (plaintext, local only)"
        );
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
        assert_eq!(
            config.delivery,
            Delivery::Smtp {
                host: "smtp-relay.example".into(),
                port: 587,
                security: SmtpSecurity::StartTls {
                    username: "login".into(),
                    password: "secret".into(),
                },
            }
        );
        let described = config.describe();
        assert_eq!(
            described,
            "mail relay smtp-relay.example:587 (STARTTLS with login)"
        );
        assert!(!described.contains("secret"));
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

    #[test]
    fn the_brevo_api_needs_its_key_and_ignores_the_smtp_variables() {
        let config = MailConfig::from_lookup(lookup(&[
            ("MAIL_TRANSPORT", "brevo-api"),
            ("BREVO_API_KEY", "xkeysib-test"),
            ("SMTP_TLS", "nonsense-that-would-fail-under-smtp"),
        ]))
        .expect("a key is all the API path needs");
        assert_eq!(
            config.delivery,
            Delivery::BrevoApi {
                api_key: "xkeysib-test".into()
            }
        );
        let described = config.describe();
        assert_eq!(described, "mail via the Brevo HTTP API (key configured)");
        assert!(!described.contains("xkeysib"));

        let error = MailConfig::from_lookup(lookup(&[("MAIL_TRANSPORT", "brevo-api")]))
            .expect_err("no key, no API path");
        assert_eq!(
            error.to_string(),
            "MAIL_TRANSPORT=brevo-api requires BREVO_API_KEY"
        );
        let error = MailConfig::from_lookup(lookup(&[("MAIL_TRANSPORT", "sendgrid")]))
            .expect_err("only the two transports are known");
        assert_eq!(
            error.to_string(),
            "MAIL_TRANSPORT must be smtp or brevo-api"
        );
    }

    #[test]
    fn the_brevo_request_carries_sender_name_recipient_subject_and_text() {
        let sender: Mailbox = "DAC2 <no-reply@dac2.local>".parse().unwrap();
        let recipient: Mailbox = "owner@example.test".parse().unwrap();
        let request = brevo_request(
            &sender,
            &recipient,
            "ยืนยันอีเมล DAC2",
            "กดลิงก์นี้\n\nhttps://x/y?token=t\n",
        );
        assert_eq!(
            request,
            serde_json::json!({
                "sender": { "name": "DAC2", "email": "no-reply@dac2.local" },
                "to": [{ "email": "owner@example.test" }],
                "subject": "ยืนยันอีเมล DAC2",
                "textContent": "กดลิงก์นี้\n\nhttps://x/y?token=t\n",
            })
        );
        let bare: Mailbox = "no-reply@dac2.local".parse().unwrap();
        assert_eq!(
            brevo_request(&bare, &recipient, "s", "t")["sender"],
            serde_json::json!({ "email": "no-reply@dac2.local" })
        );
    }
}
