//! A document's data imports (spec §8.6): what a `from` may interpolate (F031, F014), and where remote data may come
//! from (F028). A pack's imports too, since its data is loaded for its definitions; only the processed document's
//! `from` sees inputs.

use super::Checker;
use super::scope::Scope;
use crate::diagnostic::{Code, Location};
use crate::parse::params::interpolations;

/// What comes first in a `from`, which must be written as is: a URL's scheme and host, a relative path's `./`, a
/// `host:` source's prefix, a pack's name. Its length in code points; for a `from` of no known form, all of it.
fn literal_prefix(from: &[char]) -> usize {
    let text: String = from.iter().collect();
    if let Some(scheme_end) = text.find("://") {
        let host_start = text[..scheme_end + 3].chars().count();
        let host_len = from[host_start..]
            .iter()
            .take_while(|c| !matches!(c, '/' | '?' | '#'))
            .count();
        return host_start + host_len;
    }
    let prefix = ["./", "../", "host:"]
        .into_iter()
        .find(|prefix| text.starts_with(prefix));
    if let Some(prefix) = prefix {
        return prefix.len();
    }
    if text.starts_with('@') {
        let slashes = from.iter().enumerate().filter(|(_, c)| **c == '/').nth(1);
        return slashes.map_or(from.len(), |(index, _)| index + 1);
    }
    from.len()
}

/// A remote `from`'s origin, its scheme and host, when they are written as is.
fn origin(from: &str) -> Option<&str> {
    let scheme_end = from.find("://")?;
    let host_end = from[scheme_end + 3..]
        .find(['/', '?', '#'])
        .map_or(from.len(), |end| scheme_end + 3 + end);
    let origin = &from[..host_end];
    (!origin.contains("{{")).then_some(origin)
}

impl Checker<'_, '_> {
    pub(super) fn data(&mut self, allowed_origins: &[String]) {
        let resolved = self.resolved;
        let settings = &resolved.document(self.id).settings;
        let scope = if self.id == resolved.main {
            Scope::document(settings)
        } else {
            Scope::definition(settings, None)
        };
        for (index, import) in settings.data.iter().enumerate() {
            let chars: Vec<char> = import.from.value.chars().collect();
            let literal = literal_prefix(&chars);
            let mut loadable = true;
            for (at_char, path) in interpolations(&chars) {
                let at = Location::new(import.text_at.line, import.text_at.column + at_char);
                loadable &= self.data_path(&path, at_char < literal, at, &scope);
            }
            let from = &import.from.value;
            if let Some(origin) = origin(from)
                .filter(|origin| !allowed_origins.iter().any(|allowed| allowed == origin))
            {
                self.report(
                    Code::F028,
                    import.from.at,
                    format!("this processor isn't configured to fetch from `{origin}`; the data is its `fallback`, or null"),
                );
                loadable = false;
            }
            if !loadable {
                self.refuse_data(index);
            }
        }
    }

    /// An interpolation in a `from`: only `md.input` and `md.doc`, never in a URL's scheme or host, and never an input
    /// a host-language component can change (F031); whether it may be loaded.
    fn data_path(
        &mut self,
        path: &str,
        in_literal_prefix: bool,
        at: Location,
        scope: &Scope,
    ) -> bool {
        let why = if in_literal_prefix {
            "the start of a `from`, a URL's scheme and host, is written as is, so the origin can be checked"
        } else if !(path.starts_with("md.input.") || path.starts_with("md.doc.")) {
            "a `from` may interpolate only `md.input` and `md.doc`, so data from one source can't be sent to another"
        } else if path.strip_prefix("md.input.").is_some_and(|input| {
            self.bound_by_hosts
                .contains(input.split('.').next().unwrap_or(input))
        }) {
            "a host-language component can change this input, so its code could send data out through it"
        } else {
            self.path(path, at, scope);
            return scope.knows(path);
        };
        self.report(Code::F031, at, why);
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prefix(from: &str) -> usize {
        literal_prefix(&from.chars().collect::<Vec<_>>())
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
    }
}
