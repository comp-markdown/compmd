//! Interpolation (spec §8.1): a value written as text, a slot placed where a body says, and URLs with their values
//! percent-encoded; an interpolation that never gives a URL its scheme (W109). Adjacent text is joined.

use super::Expander;
use super::env::Env;
use super::origins;
use crate::cel::Value;
use crate::diagnostic::{Code, Location};
use crate::expanded::{self, Content};
use crate::parse::params::interpolations;
use crate::tree::{self, Image, Interpolation, Link, Node, Point, StringPart};
use std::fmt::Write as _;

/// The schemes an interpolated URL may have (spec §8.1).
const SCHEMES: [&str; 3] = ["https", "http", "mailto"];

impl<'r> Expander<'r> {
    /// An interpolation in a body: `md.slot` places slot content in a definition that declares the slot; any other
    /// path, its value as text, in a paragraph of its own when it stands alone.
    pub(super) fn interpolation(
        &mut self,
        interpolation: &'r Interpolation,
        env: &Env<'r>,
        out: &mut Vec<expanded::Node>,
    ) {
        let path = interpolation.path.as_str();
        if let Some(slot) = slot_name(path)
            && env.scope.knows(path)
        {
            return self.slot(slot, interpolation.block, env, out);
        }
        let start = interpolation
            .position
            .map(|position| position.start)
            .unwrap_or_default();
        let Some(text) = self.path_text(path, start, env) else {
            return;
        };
        let text = self.made(Content::Text(super::literal(&text)), None);
        if interpolation.block {
            let paragraph = Content::Paragraph(tree::Parent {
                children: vec![text],
                position: None,
            });
            let origin = env
                .origin
                .or_else(|| origins::range(&Node::Interpolation(interpolation.clone())));
            let made = self.made(paragraph, origin);
            out.push(made);
        } else {
            out.push(text);
        }
    }

    /// The text a path writes, `None` for `null`: an unknown path is shown as written (F014, from validating), and a
    /// value that can't be written, F024 at the path.
    pub(super) fn path_text(&mut self, path: &str, start: Point, env: &Env<'r>) -> Option<String> {
        let written = format!("{{{{{path}}}}}");
        if !env.scope.knows(path) {
            return Some(written);
        }
        let at = self.path_at(start, env);
        match self.evaluate(path, env) {
            Ok(value) => match value.written() {
                Ok(text) => text,
                Err(why) => {
                    self.report(
                        Code::F024,
                        env,
                        at,
                        format!("`{path}` can't be written: {why}"),
                    );
                    Some(written)
                }
            },
            Err(error) => {
                self.report(
                    Code::F024,
                    env,
                    at,
                    format!("`{path}`: {}", error.message()),
                );
                Some(written)
            }
        }
    }

    /// Where the path of the interpolation that starts at `start` is.
    fn path_at(&self, start: Point, env: &Env) -> Location {
        let found = interpolations(&self.lines[&env.document].from(start));
        found.first().map_or(start.into(), |(index, _)| {
            Location::new(start.line, start.column + index)
        })
    }

    /// A slot placed by `{{md.slot}}` or `{{md.slot.<name>}}`: its content, expanded in the caller's environment; a
    /// template slot once per element (spec §7.5). Inline, a slot that is one paragraph gives its text.
    fn slot(&mut self, name: &str, block: bool, env: &Env<'r>, out: &mut Vec<expanded::Node>) {
        let Some(slots) = env.slots.clone() else {
            return;
        };
        let content = if name == "default" {
            slots.default
        } else {
            slots
                .named
                .iter()
                .find(|slot| slot.name == name)
                .map_or(&[][..], |slot| slot.children.as_slice())
        };
        let mut placed = Vec::new();
        match slots
            .templates
            .iter()
            .find(|template| template.slot == name)
        {
            Some(template) => {
                for index in 0..template.items.len() {
                    if self.over_limit() {
                        break;
                    }
                    let inner = slots.env.iteration(&template.name, &template.items, index);
                    placed.extend(self.list(content, &inner));
                }
            }
            None => placed = self.list(content, &slots.env),
        }
        match placed.as_mut_slice() {
            [single] if !block && matches!(single.content, Content::Paragraph(_)) => {
                out.extend(
                    single
                        .content
                        .children_mut()
                        .map(std::mem::take)
                        .unwrap_or_default(),
                );
            }
            _ => out.extend(placed),
        }
    }

    /// A link; one whose interpolated URL would get a scheme it may not have is its text alone (W109).
    pub(super) fn link(
        &mut self,
        link: &'r Link,
        node: &'r Node,
        env: &Env<'r>,
        out: &mut Vec<expanded::Node>,
    ) {
        let children = self.list(&link.children, env);
        match self.checked_url(&link.url, link.url_parts.as_deref(), node, env) {
            Some(url) => {
                let content = Content::Link(Link {
                    children,
                    url,
                    title: link.title.clone(),
                    url_parts: None,
                    position: None,
                });
                let made = self.made(content, origins::of(node, env));
                out.push(made);
            }
            None => out.extend(children),
        }
    }

    /// An image; one whose interpolated URL would get a scheme it may not have is its alternative text (W109).
    pub(super) fn image(
        &mut self,
        image: &'r Image,
        node: &'r Node,
        env: &Env<'r>,
        out: &mut Vec<expanded::Node>,
    ) {
        let content = match self.checked_url(&image.url, image.url_parts.as_deref(), node, env) {
            Some(url) => Content::Image(Image {
                url,
                url_parts: None,
                position: None,
                ..image.clone()
            }),
            None => Content::Text(super::literal(&image.alt)),
        };
        let made = self.made(content, origins::of(node, env));
        out.push(made);
    }

    /// A URL with its interpolations filled in; `None`, with W109, when one at its start gives it a scheme other
    /// than `https:`, `http:` or `mailto:`.
    pub(super) fn checked_url(
        &mut self,
        written: &str,
        parts: Option<&[StringPart]>,
        node: &Node,
        env: &Env<'r>,
    ) -> Option<String> {
        let url = self.url(written, parts, env);
        let interpolated_start = matches!(parts, Some([StringPart::Interpolation(_), ..]));
        let scheme = scheme(&url);
        if interpolated_start && scheme.is_some_and(|scheme| !SCHEMES.contains(&scheme.as_str())) {
            let at = node
                .position()
                .map_or(Location::new(1, 1), |p| p.start.into());
            self.report(
                Code::W109,
                env,
                at,
                "an interpolated URL may only be relative or `https:`, `http:` or `mailto:`; the link's text is shown without the link",
            );
            return None;
        }
        Some(url)
    }

    /// A URL with its interpolations filled in, each value percent-encoded where a URL can't hold it.
    fn url(&mut self, written: &str, parts: Option<&[StringPart]>, env: &Env<'r>) -> String {
        let Some(parts) = parts else {
            return written.to_string();
        };
        let mut url = String::new();
        for part in parts {
            match part {
                StringPart::Text(text) => url.push_str(&text.value),
                StringPart::Interpolation(interpolation) => {
                    let path = &interpolation.path;
                    let value = if env.scope.knows(path) {
                        self.evaluate(path, env).ok()
                    } else {
                        None
                    };
                    match value.as_ref().map(Value::written) {
                        Some(Ok(text)) => url.push_str(&url_safe(&text.unwrap_or_default())),
                        _ => {
                            let _ = write!(url, "{{{{{path}}}}}");
                        }
                    }
                }
            }
        }
        url
    }
}

/// The slot `{{md.slot}}` or `{{md.slot.<name>}}` names.
fn slot_name(path: &str) -> Option<&str> {
    match path {
        "md.slot" => Some("default"),
        _ => path.strip_prefix("md.slot."),
    }
}

/// A URL's scheme, lowercased, when it has one.
fn scheme(url: &str) -> Option<String> {
    let (scheme, _) = url.split_once(':')?;
    let mut chars = scheme.chars();
    let valid = chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'));
    valid.then(|| scheme.to_ascii_lowercase())
}

/// `text` with the characters a URL can't hold percent-encoded; `/`, `?` and `#` are kept.
fn url_safe(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        if c.is_ascii_alphanumeric() || "-._~:/?#[]@!$&'()*+,;=%".contains(c) {
            out.push(c);
        } else {
            let mut bytes = [0; 4];
            for byte in c.encode_utf8(&mut bytes).bytes() {
                let _ = write!(out, "%{byte:02X}");
            }
        }
    }
    out
}

/// Adjacent text nodes joined into one; the joined node keeps an origin only when all its parts had the same.
pub(super) fn joined(nodes: Vec<expanded::Node>) -> Vec<expanded::Node> {
    let mut out: Vec<expanded::Node> = Vec::with_capacity(nodes.len());
    for node in nodes {
        if let (Some(last), Content::Text(text)) = (out.last_mut(), &node.content)
            && let Content::Text(joined) = &mut last.content
        {
            joined.value.push_str(&text.value);
            if last.origin != node.origin {
                last.origin = None;
            }
            continue;
        }
        out.push(node);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_keep_their_structure() {
        assert_eq!(url_safe("acme/ray play?q=1#x"), "acme/ray%20play?q=1#x");
        assert_eq!(url_safe("é\"<>"), "%C3%A9%22%3C%3E");
    }

    #[test]
    fn schemes_are_letters_before_a_colon() {
        assert_eq!(scheme("JavaScript:alert(1)"), Some("javascript".into()));
        assert_eq!(scheme("https://a.org"), Some("https".into()));
        assert_eq!(scheme("./a:b"), None);
        assert_eq!(scheme("1a:b"), None);
        assert_eq!(scheme("docs/page"), None);
    }

    #[test]
    fn slots_are_named_by_their_path() {
        assert_eq!(slot_name("md.slot"), Some("default"));
        assert_eq!(slot_name("md.slot.footer"), Some("footer"));
        assert_eq!(slot_name("md.doc"), None);
    }
}
