//! A data import's `from` (spec §8.6): its interpolations filled in, each value percent-encoded as one path segment
//! or one query value, and the source it names.

use crate::cel::{Key, Value};
use crate::parse::params::string_parts;
use crate::resolve::{DocumentId, Origin, Resolved};
use crate::settings::DataImport;
use crate::tree::StringPart;
use std::fmt::Write as _;
use std::rc::Rc;

/// The source a `from` names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Target {
    /// A file of the project or of a pack.
    File(Origin),
    /// An `https:` URL.
    Remote(String),
    /// A `host:` source, without `host:`.
    Host(String),
}

/// The source of `import`, written in the document `id`, with `input` as `md.input` of the processed document.
pub(super) fn target(
    resolved: &Resolved,
    id: DocumentId,
    import: &DataImport,
    input: &Value,
) -> Result<Target, String> {
    let document = resolved.document(id);
    let doc = Value::from_json(&serde_json::Value::Object(document.settings.doc.clone()));
    let input = if id == resolved.main {
        input
    } else {
        &Value::Null
    };
    let mut from = String::new();
    for part in string_parts(&import.from.value) {
        match part {
            StringPart::Text(text) => from.push_str(&text.value),
            StringPart::Interpolation(interpolation) => {
                let value = lookup(&interpolation.path, &doc, input);
                let text = value.written().ok().flatten().unwrap_or_default();
                from.push_str(&encoded(&text));
            }
        }
    }
    if let Some(source) = from.strip_prefix("host:") {
        return Ok(Target::Host(source.to_string()));
    }
    if from.contains("://") {
        return if from.starts_with("https://") {
            Ok(Target::Remote(from))
        } else {
            Err(format!("`{from}` isn't an `https:` URL"))
        };
    }
    if from.starts_with('@') {
        return pack_file(resolved, &from);
    }
    document
        .origin
        .join(&from)
        .map(Target::File)
        .ok_or_else(|| format!("`{from}` leaves its folder"))
}

/// A file inside an imported pack, `@scope/pack/path`, in the release that was imported.
fn pack_file(resolved: &Resolved, from: &str) -> Result<Target, String> {
    let mut segments = from.splitn(3, '/');
    let pack = format!(
        "{}/{}",
        segments.next().unwrap_or_default(),
        segments.next().unwrap_or_default()
    );
    let path = segments.next().unwrap_or_default();
    resolved
        .documents
        .iter()
        .find_map(|document| match &document.origin {
            Origin::Pack { name, version, .. } if *name == pack => Some(Origin::Pack {
                name: name.clone(),
                version: version.clone(),
                path: path.to_string(),
            }),
            _ => None,
        })
        .map(Target::File)
        .ok_or_else(|| format!("`{pack}` isn't imported, so `{from}` can't be loaded"))
}

/// The value of a `md.doc` or `md.input` path; `null` for anything else, which validating refuses.
fn lookup(path: &str, doc: &Value, input: &Value) -> Value {
    let mut segments = path.split('.');
    let mut value = match (segments.next(), segments.next()) {
        (Some("md"), Some("doc")) => doc.clone(),
        (Some("md"), Some("input")) => input.clone(),
        _ => return Value::Null,
    };
    for segment in segments {
        value = match &value {
            Value::Map(fields) => fields
                .get(&Key::String(Rc::from(segment)))
                .cloned()
                .unwrap_or(Value::Null),
            _ => Value::Null,
        };
    }
    value
}

/// `text` percent-encoded as one path segment or one query value: it can't add `/`, `..`, `?`, `&` or `#`.
pub(super) fn encoded(text: &str) -> String {
    let dots_only = !text.is_empty() && text.chars().all(|c| c == '.');
    let mut out = String::new();
    for byte in text.bytes() {
        let kept = byte.is_ascii_alphanumeric()
            || matches!(byte, b'-' | b'_' | b'~')
            || (byte == b'.' && !dots_only);
        if kept {
            out.push(char::from(byte));
        } else {
            let _ = write!(out, "%{byte:02X}");
        }
    }
    out
}

/// The path of a URL, without its query and fragment, for its extension.
pub(super) fn path_of(url: &str) -> &str {
    url.split(['?', '#']).next().unwrap_or(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_stay_in_their_segment() {
        assert_eq!(encoded("eu west"), "eu%20west");
        assert_eq!(encoded("a/b?c&d#e"), "a%2Fb%3Fc%26d%23e");
        assert_eq!(encoded(".."), "%2E%2E");
        assert_eq!(encoded("v1.2"), "v1.2");
        assert_eq!(encoded("é"), "%C3%A9");
        assert_eq!(encoded(""), "");
    }

    #[test]
    fn paths_lose_their_query_and_fragment() {
        assert_eq!(path_of("https://a.org/x.csv?y=1"), "https://a.org/x.csv");
        assert_eq!(path_of("https://a.org/x.json#top"), "https://a.org/x.json");
    }
}
