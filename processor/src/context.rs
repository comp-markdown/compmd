//! The render context (spec §8.5): what a render depends on besides the document, its imports and its data.

use crate::cel::time::parse_timestamp;
use crate::cel::{Key, Value};
use crate::host::Time;
use crate::md::Md;
use crate::settings::Settings;
use indexmap::IndexMap;
use serde_json::{Map, Value as Json};
use std::rc::Rc;

/// The render time, the processor's time zone, the host's time zone offsets and the input values one render uses.
/// The same context gives the same expanded tree.
pub struct Context<'a> {
    /// The render time, in nanoseconds since 1970-01-01T00:00:00Z: what `md.now()` and `md.today()` give.
    pub now: i128,
    /// The processor's time zone, `md.tz` for a document without a `tz` key: an IANA name, or `UTC`.
    pub tz: String,
    /// The host's time zone offsets, for `md.tz` and the zones expressions name.
    pub time: &'a dyn Time,
    /// The values given for the document's inputs, by name: from a command line, a URL query, a host. A value that
    /// doesn't fit its input's type is ignored, and the default is used (spec §8.7).
    pub inputs: Map<String, Json>,
}

/// A render time for [`Context::now`] from RFC 3339 text, such as `2026-10-21T10:00:00Z` or
/// `2026-10-21T12:00:00.5+02:00`: nanoseconds since 1970-01-01T00:00:00Z, or `None` when the text isn't one or is
/// outside CEL's range (years 1 to 9999).
///
/// ```
/// assert_eq!(compmd::context::parse_time("1970-01-01T00:00:01Z"), Some(1_000_000_000));
/// assert_eq!(compmd::context::parse_time("yesterday"), None);
/// ```
#[must_use]
pub fn parse_time(text: &str) -> Option<i128> {
    parse_timestamp(text)
}

/// `md.lang` of a document without a `lang` key (spec §2.1).
const DEFAULT_LANG: &str = "en";

impl Context<'_> {
    /// The `md.` functions for expressions written in a document with `settings`: its language, and its time zone
    /// or the processor's.
    pub(crate) fn functions(&self, settings: &Settings) -> Md<'_> {
        Md::new(
            self.now,
            settings.lang.as_deref().unwrap_or(DEFAULT_LANG),
            self.tz(settings),
            self.time,
        )
    }

    /// `md` of a document with `settings`, as far as the document alone gives it: `doc`, `version`, `lang` and `tz`
    /// (spec §8.1). A render adds `data`, `input` and `loop`.
    pub(crate) fn md(&self, settings: &Settings) -> IndexMap<Key, Value> {
        let text = |text: Option<&str>| text.map_or(Value::Null, Value::string);
        IndexMap::from([
            (
                key("doc"),
                Value::from_json(&Json::Object(settings.doc.clone())),
            ),
            (key("version"), text(settings.version.as_deref())),
            (
                key("lang"),
                Value::string(settings.lang.as_deref().unwrap_or(DEFAULT_LANG)),
            ),
            (key("tz"), Value::string(self.tz(settings))),
        ])
    }

    fn tz<'s>(&'s self, settings: &'s Settings) -> &'s str {
        settings.tz.as_deref().unwrap_or(&self.tz)
    }
}

/// A string key of `md`.
pub(crate) fn key(name: &str) -> Key {
    Key::String(Rc::from(name))
}
