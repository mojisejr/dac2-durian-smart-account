//! Reading the process environment at start. Each rule is a pure function
//! over the value found, so the binary applies it and the tests exercise it
//! without touching the real environment.

/// A boolean variable is unset, `true`, or `false`. Anything else is a
/// configuration mistake worth stopping on rather than a value to guess at.
pub fn flag(name: &str, value: Option<String>) -> Result<bool, String> {
    match value.as_deref().map(str::trim) {
        None | Some("") | Some("false") => Ok(false),
        Some("true") => Ok(true),
        Some(_) => Err(format!("{name} must be true or false")),
    }
}

/// `PORT`, when a host sets it, replaces the port of the configured address
/// and nothing else.
pub fn port(value: Option<String>) -> Result<Option<u16>, String> {
    match value.as_deref().map(str::trim) {
        None | Some("") => Ok(None),
        Some(text) => text
            .parse()
            .map(Some)
            .map_err(|error| format!("PORT must be a port number: {error}")),
    }
}

/// A variable the server cannot run without. The error names the variable
/// and never repeats its value.
pub fn required(name: &str, value: Option<String>) -> Result<String, String> {
    match value {
        Some(value) if !value.trim().is_empty() => Ok(value),
        _ => Err(format!("{name} must be set")),
    }
}

/// A session cookie that travels over HTTPS must be marked Secure, or a
/// downgraded request leaks it. The server refuses the combination rather
/// than serving with it.
pub fn https_requires_secure_cookie(base_url: &str, cookie_secure: bool) -> Result<(), String> {
    if base_url
        .trim_start()
        .to_ascii_lowercase()
        .starts_with("https://")
        && !cookie_secure
    {
        return Err("APP_BASE_URL is https:// so COOKIE_SECURE must be true".into());
    }
    Ok(())
}

/// Whether the shell shows the pilot notice. Set once at start from
/// PILOT_NOTICE and read by every render.
static PILOT_NOTICE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

pub fn set_pilot_notice(shown: bool) {
    let _ = PILOT_NOTICE.set(shown);
}

pub fn pilot_notice() -> bool {
    PILOT_NOTICE.get().copied().unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_required_variable_is_named_when_missing_or_blank() {
        assert_eq!(
            required("DATABASE_URL", None),
            Err("DATABASE_URL must be set".into())
        );
        assert_eq!(
            required("SESSION_KEY", Some("   ".into())),
            Err("SESSION_KEY must be set".into())
        );
        assert_eq!(
            required("DATABASE_URL", Some("postgres://x".into())),
            Ok("postgres://x".into())
        );
    }

    #[test]
    fn an_https_base_url_demands_a_secure_cookie() {
        assert_eq!(
            https_requires_secure_cookie("http://127.0.0.1:3000", false),
            Ok(())
        );
        assert_eq!(
            https_requires_secure_cookie("https://dac2.example", true),
            Ok(())
        );
        assert_eq!(
            https_requires_secure_cookie("HTTPS://dac2.example", false),
            Err("APP_BASE_URL is https:// so COOKIE_SECURE must be true".into())
        );
    }

    #[test]
    fn an_unset_or_blank_flag_is_off() {
        assert_eq!(flag("COOKIE_SECURE", None), Ok(false));
        assert_eq!(flag("COOKIE_SECURE", Some("  ".into())), Ok(false));
    }

    #[test]
    fn only_the_words_true_and_false_are_accepted() {
        assert_eq!(flag("COOKIE_SECURE", Some("true".into())), Ok(true));
        assert_eq!(flag("COOKIE_SECURE", Some(" false ".into())), Ok(false));
        assert_eq!(
            flag("COOKIE_SECURE", Some("yes".into())),
            Err("COOKIE_SECURE must be true or false".into())
        );
    }

    #[test]
    fn a_host_port_is_a_number_or_absent() {
        assert_eq!(port(None), Ok(None));
        assert_eq!(port(Some("10000".into())), Ok(Some(10000)));
        assert!(port(Some("http".into())).is_err());
    }
}
