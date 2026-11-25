//! What is wrong with a configuration.

use std::fmt;

/// Every problem found in a configuration, each saying where it is (`packs["./components"].trust: …`), and where the
/// configuration came from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigError {
    /// Where the configuration came from: `compmd.config.json`, or `package.json (`compmd`)`.
    pub source: String,
    /// The problems, in the order the configuration writes what they are about.
    pub problems: Vec<String>,
}

/// `compmd.config.json:`, then each problem on a line of its own, as `@compmd/vite` writes it.
impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:", self.source)?;
        for problem in &self.problems {
            write!(formatter, "\n- {problem}")?;
        }
        Ok(())
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn says_where_and_each_problem_on_a_line() {
        let error = ConfigError {
            source: "compmd.config.json".into(),
            problems: vec![
                "imports: `any` or `listed`".into(),
                "fetch: `host` or `direct`".into(),
            ],
        };
        assert_eq!(
            error.to_string(),
            "compmd.config.json:\n- imports: `any` or `listed`\n- fetch: `host` or `direct`"
        );
    }
}
