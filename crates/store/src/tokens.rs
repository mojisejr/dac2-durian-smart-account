use rand::random;
use sha2::{Digest, Sha256};

pub struct IssuedToken(String);

impl IssuedToken {
    pub fn expose(&self) -> &str {
        &self.0
    }
}

pub fn issue() -> IssuedToken {
    let bytes = random::<[u8; 32]>();
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write;
        write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
    }
    IssuedToken(encoded)
}

pub fn hash(raw: &str) -> Vec<u8> {
    Sha256::digest(raw.as_bytes()).to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_has_32_random_bytes_and_is_not_debug_printable() {
        let first = issue();
        let second = issue();
        assert_eq!(first.expose().len(), 64);
        assert!(first.expose().bytes().all(|byte| byte.is_ascii_hexdigit()));
        assert_ne!(first.expose(), second.expose());
    }

    #[test]
    fn hash_is_stable_and_an_altered_token_does_not_match() {
        let token = issue();
        assert_eq!(hash(token.expose()), hash(token.expose()));
        assert_ne!(hash(token.expose()), hash("altered"));
        assert_eq!(hash(token.expose()).len(), 32);
    }
}
