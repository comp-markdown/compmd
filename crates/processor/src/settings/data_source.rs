//! Where a data import's `from` reads (spec §8.6), told once from how it is written (audit I6, A6): validating checks
//! what it may interpolate and where it may fetch from by it, and loading fills its interpolations in. A value an
//! interpolation gives is percent-encoded as one segment (`load/from.rs`), so it can never make one kind another.

/// The kind of source a `from` names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DataSource {
    /// A file of the root folder, by a relative path: `./sales.csv`.
    File,
    /// A file inside an imported pack: `@acme/kit/icons.json`.
    Pack,
    /// A URL: `https://api.example.com/x.json`.
    Remote,
    /// A source the host provides: `host:tasks`.
    Host,
}

impl DataSource {
    /// The kind `from`, as written, names.
    pub(crate) fn of(from: &str) -> Self {
        if from.starts_with("host:") {
            Self::Host
        } else if from.contains("://") {
            Self::Remote
        } else if from.starts_with('@') {
            Self::Pack
        } else {
            Self::File
        }
    }

    /// What comes first in `from`, which must be written as is: a URL's scheme and host, a `host:` source's prefix, a
    /// pack's name, a relative path's `./` or `../`. Its length in code points; for a path of no known form, all of it.
    pub(crate) fn literal_prefix(self, from: &[char]) -> usize {
        let text: String = from.iter().collect();
        match self {
            Self::Remote => {
                let host_start = text
                    .find("://")
                    .map_or(0, |scheme_end| text[..scheme_end + 3].chars().count());
                let host_len = from[host_start..]
                    .iter()
                    .take_while(|c| !matches!(c, '/' | '?' | '#'))
                    .count();
                host_start + host_len
            }
            Self::Host => "host:".len(),
            Self::Pack => {
                let second_slash = from.iter().enumerate().filter(|(_, c)| **c == '/').nth(1);
                second_slash.map_or(from.len(), |(index, _)| index + 1)
            }
            Self::File => ["./", "../"]
                .into_iter()
                .find(|prefix| text.starts_with(prefix))
                .map_or(from.len(), str::len),
        }
    }

    /// A remote `from`'s origin, its scheme and host, when they are written as is; none for another kind.
    pub(crate) fn origin(self, from: &str) -> Option<&str> {
        if self != Self::Remote {
            return None;
        }
        let scheme_end = from.find("://")?;
        let host_end = from[scheme_end + 3..]
            .find(['/', '?', '#'])
            .map_or(from.len(), |end| scheme_end + 3 + end);
        let origin = &from[..host_end];
        (!origin.contains("{{")).then_some(origin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prefix(from: &str) -> usize {
        DataSource::of(from).literal_prefix(&from.chars().collect::<Vec<_>>())
    }

    #[test]
    fn a_from_is_one_kind_of_source() {
        assert_eq!(DataSource::of("./sales.csv"), DataSource::File);
        assert_eq!(DataSource::of("sales.csv"), DataSource::File);
        assert_eq!(DataSource::of("@acme/kit/icons.json"), DataSource::Pack);
        assert_eq!(
            DataSource::of("https://api.example.com/x"),
            DataSource::Remote
        );
        assert_eq!(
            DataSource::of("http://api.example.com/x"),
            DataSource::Remote
        );
        assert_eq!(DataSource::of("host:tasks"), DataSource::Host);
        assert_eq!(
            DataSource::of("host:a://b"),
            DataSource::Host,
            "a host source first"
        );
    }

    #[test]
    fn the_start_of_a_from_is_literal() {
        assert_eq!(prefix("https://api.example.com/x?{{a}}"), 23);
        assert_eq!(prefix("https://{{a}}.example.com/x"), 25);
        assert_eq!(prefix("./data-{{a}}.csv"), 2);
        assert_eq!(prefix("../x.csv"), 3);
        assert_eq!(prefix("host:tasks?p={{a}}"), 5);
        assert_eq!(prefix("@acme/kit/{{a}}.json"), 10);
        assert_eq!(prefix("@acme"), 5);
        assert_eq!(prefix("{{a}}"), 5);
    }

    #[test]
    fn origins_are_scheme_and_host() {
        fn origin(from: &str) -> Option<&str> {
            DataSource::of(from).origin(from)
        }
        assert_eq!(
            origin("https://api.example.com/x"),
            Some("https://api.example.com")
        );
        assert_eq!(
            origin("https://api.example.com"),
            Some("https://api.example.com")
        );
        assert_eq!(origin("https://{{a}}.example.com/x"), None);
        assert_eq!(origin("./x.csv"), None);
        assert_eq!(
            origin("host:a://b"),
            None,
            "a host source fetches nothing itself"
        );
    }
}
