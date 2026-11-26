//! The token in the preview page's path (docs/editor-support.md, decision 8): 128 bits from the system's randomness, so
//! only the editor that opened the page, which the server gave the path, can reach it.

use std::fmt::Write as _;

/// A new token: 32 hexadecimal digits.
///
/// # Errors
///
/// When the system gives no randomness.
pub(crate) fn token() -> Result<String, getrandom::Error> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes)?;
    Ok(bytes.iter().fold(String::new(), |mut token, byte| {
        let _ = write!(token, "{byte:02x}");
        token
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_token_is_32_hexadecimal_digits_and_new_each_time() {
        let (first, second) = (token().expect("a token"), token().expect("a token"));
        assert_eq!(first.len(), 32);
        assert!(first.bytes().all(|byte| byte.is_ascii_hexdigit()));
        assert_ne!(first, second);
    }
}
