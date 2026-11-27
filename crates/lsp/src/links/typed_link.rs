//! What is being typed in a link ([the language server], *Links and headings*, completion), read from the line before
//! the cursor: a link that isn't finished yet has no node in the parse tree.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#links-and-headings

/// A link's part the cursor is at the end of, with what is written of it so far.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TypedLink {
    /// A destination: after `](`, or a reference definition's after `[label]: `.
    Target(String),
    /// A reference's label, after `[text][`.
    Label(String),
}

/// What is being typed at the end of `before`, the line's text up to the cursor; `None` outside a link.
pub(crate) fn typed_link(before: &str) -> Option<TypedLink> {
    if let Some((_, written)) = before.rsplit_once("](")
        && !written.contains([')', ' ', '\t'])
    {
        return Some(TypedLink::Target(written.to_owned()));
    }
    if let Some(written) = definition_target(before) {
        return Some(TypedLink::Target(written.to_owned()));
    }
    let (_, written) = before.rsplit_once("][")?;
    (!written.contains(']')).then(|| TypedLink::Label(written.to_owned()))
}

/// What is written of a reference definition's destination, when `before` is one up to it: at most three spaces,
/// `[label]:`, spaces, then the destination so far.
fn definition_target(before: &str) -> Option<&str> {
    let line = before
        .strip_prefix("   ")
        .or_else(|| before.strip_prefix("  "))
        .or_else(|| before.strip_prefix(' '))
        .unwrap_or(before);
    let rest = line.strip_prefix('[')?;
    let (label, rest) = rest.split_once("]:")?;
    if label.is_empty() || label.contains(['[', ']']) {
        return None;
    }
    let written = rest.trim_start_matches([' ', '\t']);
    (written.len() < rest.len() && !written.contains([' ', '\t'])).then_some(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(written: &str) -> TypedLink {
        TypedLink::Target(written.into())
    }

    #[test]
    fn after_a_links_parenthesis_or_a_definitions_colon_is_its_target() {
        assert_eq!(typed_link("See [the tides]("), Some(target("")));
        assert_eq!(typed_link("See [the tides](../tid"), Some(target("../tid")));
        assert_eq!(
            typed_link("![a map](maps/harbour.md#hig"),
            Some(target("maps/harbour.md#hig"))
        );
        assert_eq!(typed_link("[tides]: ./ti"), Some(target("./ti")));
        assert_eq!(typed_link("  [tides]: "), Some(target("")));
    }

    #[test]
    fn after_a_references_bracket_is_its_label() {
        assert_eq!(
            typed_link("See [the tides][ti"),
            Some(TypedLink::Label("ti".into()))
        );
        assert_eq!(typed_link("[a][]"), None, "a collapsed reference, closed");
    }

    #[test]
    fn anything_else_is_no_link() {
        for before in [
            "Plain text",
            "[the tides](tides.md) and",
            "[the tides](tides.md \"Title",
            "[tides]:./tides.md",
            "    [tides]: ./ti",
            "[the tides][tables] and",
        ] {
            assert_eq!(typed_link(before), None, "{before}");
        }
    }
}
