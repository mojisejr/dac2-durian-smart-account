use std::{env, fmt};

use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub from: String,
    pub base_url: String,
}

impl MailConfig {
    pub fn from_env() -> Self {
        Self {
            smtp_host: nonempty_env("SMTP_HOST").unwrap_or_else(|_| "127.0.0.1".into()),
            smtp_port: nonempty_env("SMTP_PORT")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(1025),
            from: nonempty_env("MAIL_FROM").unwrap_or_else(|_| "DAC2 <no-reply@dac2.local>".into()),
            base_url: nonempty_env("APP_BASE_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:3000".into()),
        }
    }
}

fn nonempty_env(name: &str) -> Result<String, env::VarError> {
    env::var(name).and_then(|value| {
        if value.trim().is_empty() {
            Err(env::VarError::NotPresent)
        } else {
            Ok(value)
        }
    })
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
    pub fn new(config: MailConfig) -> Self {
        let transport =
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(config.smtp_host.clone())
                .port(config.smtp_port)
                .build();
        Self { config, transport }
    }

    pub fn from_env() -> Self {
        Self::new(MailConfig::from_env())
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
            from: "DAC2 <no-reply@dac2.local>".into(),
            base_url: "http://127.0.0.1:3000/".into(),
        });
        assert_eq!(
            mailer.link("auth/verify-email", "secret-token"),
            "http://127.0.0.1:3000/auth/verify-email?token=secret-token"
        );
    }
}
