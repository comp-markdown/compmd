//! The origins a document's remote data may come from (spec §8.6, §15): those the host or its user named, or every
//! origin, which a host allows only for documents it publishes itself, such as a static site's or a blog's.

/// Where a processor may fetch a document's remote data from; any other origin is F028.
///
/// ```
/// use compmd::AllowedOrigins;
///
/// let named = AllowedOrigins::from(&["https://api.example.com".to_string()]);
/// assert!(named.allows("https://api.example.com"));
/// assert!(!named.allows("https://elsewhere.example"));
/// assert!(AllowedOrigins::Every.allows("https://elsewhere.example"));
/// assert!(!AllowedOrigins::default().allows("https://api.example.com"));
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AllowedOrigins {
    /// The origins named, each `https://host[:port]`; none by default.
    Listed(Vec<String>),
    /// Every origin: no origin of a document's data is F028.
    Every,
}

impl AllowedOrigins {
    /// Whether a document's remote data may come from `origin`, `https://host[:port]`.
    #[must_use]
    pub fn allows(&self, origin: &str) -> bool {
        match self {
            Self::Listed(origins) => origins.iter().any(|allowed| allowed == origin),
            Self::Every => true,
        }
    }
}

impl Default for AllowedOrigins {
    /// No origin.
    fn default() -> Self {
        Self::Listed(Vec::new())
    }
}

impl From<&[String]> for AllowedOrigins {
    fn from(origins: &[String]) -> Self {
        Self::Listed(origins.to_vec())
    }
}

impl<const N: usize> From<&[String; N]> for AllowedOrigins {
    fn from(origins: &[String; N]) -> Self {
        Self::Listed(origins.to_vec())
    }
}

impl From<&Vec<String>> for AllowedOrigins {
    fn from(origins: &Vec<String>) -> Self {
        Self::Listed(origins.clone())
    }
}

impl From<&Self> for AllowedOrigins {
    fn from(origins: &Self) -> Self {
        origins.clone()
    }
}
