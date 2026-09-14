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

#[cfg(test)]
mod tests {
    use super::*;

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
