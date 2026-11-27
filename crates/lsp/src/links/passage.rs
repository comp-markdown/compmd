//! A link to a passage ([the language server], *Links and headings*; spec §2.3): the web's text directive,
//! `#:~:text=[prefix-,]start[,end][,-suffix]`, each part percent-decoded, matched in the target's source text. The
//! passage is the first match of `start` (through `end`) after `prefix` and before `suffix`, as the spec lets a tool
//! show it.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#links-and-headings

use percent_encoding::percent_decode_str;

/// A text directive's parts, decoded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Passage {
    prefix: Option<String>,
    start: String,
    end: Option<String>,
    suffix: Option<String>,
}

/// The passage a link's `fragment` (after `#`) names; `None` for a fragment that isn't a text directive.
pub(crate) fn passage_of(fragment: &str) -> Option<Passage> {
    let directive = fragment
        .strip_prefix(":~:")?
        .split('&')
        .find_map(|directive| directive.strip_prefix("text="))?;
    let mut parts: Vec<&str> = directive.split(',').collect();
    let prefix = match parts.first() {
        Some(first) if parts.len() > 1 && first.ends_with('-') => {
            let prefix = first.trim_end_matches('-');
            parts.remove(0);
            Some(decoded(prefix))
        }
        _ => None,
    };
    let suffix = match parts.last() {
        Some(last) if parts.len() > 1 && last.starts_with('-') => {
            let suffix = decoded(last.trim_start_matches('-'));
            parts.pop();
            Some(suffix)
        }
        _ => None,
    };
    let (start, end) = match parts[..] {
        [start] => (start, None),
        [start, end] => (start, Some(decoded(end))),
        _ => return None,
    };
    let start = decoded(start);
    (!start.is_empty()).then_some(Passage {
        prefix,
        start,
        end,
        suffix,
    })
}

impl Passage {
    /// The text it starts with.
    pub(crate) fn start(&self) -> &str {
        &self.start
    }

    /// Where the passage is in `text`, in code points: its first character and the one after its last.
    pub(crate) fn find(&self, text: &str) -> Option<(usize, usize)> {
        for (at, _) in text.match_indices(&self.start) {
            if let Some(prefix) = &self.prefix
                && !text[..at].trim_end().ends_with(prefix.as_str())
            {
                continue;
            }
            let mut end = at + self.start.len();
            if let Some(last) = &self.end {
                end += text[end..].find(last.as_str())? + last.len();
            }
            if let Some(suffix) = &self.suffix
                && !text[end..].trim_start().starts_with(suffix.as_str())
            {
                continue;
            }
            let chars = |bytes: usize| text[..bytes].chars().count();
            return Some((chars(at), chars(end)));
        }
        None
    }
}

fn decoded(part: &str) -> String {
    percent_decode_str(part).decode_utf8_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(fragment: &str, text: &str) -> Option<String> {
        let (from, to) = passage_of(fragment)?.find(text)?;
        Some(text.chars().skip(from).take(to - from).collect())
    }

    const TEXT: &str = "Café prices: 12 at noon.\n\nThe price: 12, then the price: 14 at dusk.\n";

    #[test]
    fn a_passage_is_its_starts_first_match() {
        assert_eq!(
            found(":~:text=price:%2012", TEXT).as_deref(),
            Some("price: 12")
        );
        assert_eq!(
            found(":~:text=Caf%C3%A9", TEXT).as_deref(),
            Some("Café"),
            "percent-decoded"
        );
        assert_eq!(found(":~:text=tides", TEXT), None);
        assert_eq!(
            passage_of(":~:text=a-,price,-b").map(|passage| passage.start().to_owned()),
            Some("price".into())
        );
    }

    #[test]
    fn through_its_end_after_its_prefix_and_before_its_suffix() {
        assert_eq!(
            found(":~:text=then-,the%20price,-%3A%2014", TEXT).as_deref(),
            Some("the price"),
        );
        assert_eq!(
            found(":~:text=The,dusk", TEXT).as_deref(),
            Some("The price: 12, then the price: 14 at dusk")
        );
        assert_eq!(
            found(":~:text=price:%2012,-then", TEXT).as_deref(),
            None,
            "the suffix comes after `,`"
        );
        assert_eq!(found(":~:text=price,dawn", TEXT), None);
        assert_eq!(
            found(":~:x=1&text=noon", TEXT).as_deref(),
            Some("noon"),
            "among other directives"
        );
    }

    #[test]
    fn a_fragment_that_isnt_a_text_directive_is_none() {
        for fragment in [
            "high-water",
            ":~:",
            ":~:text=",
            ":~:text=a,b,c",
            ":~:other=a",
        ] {
            assert_eq!(passage_of(fragment), None, "{fragment}");
        }
    }
}
