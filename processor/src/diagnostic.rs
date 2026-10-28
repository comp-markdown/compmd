//! Diagnostics (spec §10) in the machine form of spec §10.5.

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use std::fmt;

/// A diagnostic code from the catalog (spec §10.2): `F…` codes are errors, `W…` codes warnings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Code {
    /// Unsupported major version.
    F001,
    /// A closer names no open container while one is open.
    F003,
    /// A container isn't closed before an enclosing closer or the end of the file.
    F004,
    /// A closer while no container is open.
    F005,
    /// A named slot given twice in one container.
    F007,
    /// A malformed param list.
    F011,
    /// A continuation line that doesn't follow an opener, a leaf or a continuation line.
    F012,
    /// A slot line outside a container.
    F015,
    /// A malformed component line.
    F016,
    /// A param given twice, or `id` together with `#name`.
    F017,
    /// A closer names a container opened outside its list item or block quote.
    F032,
    /// Frontmatter that isn't YAML 1.2 core, or uses tags, anchors, aliases or repeated keys.
    F033,
    /// A param list on a signature fence.
    F036,
    /// A leaf without fallback text.
    W101,
    /// `{{` followed by something that isn't a path.
    W106,
    /// An inline component followed by a space and a param list.
    W107,
    /// A container opener or leaf directly after a paragraph line.
    W108,
    /// A later minor version than the processor implements.
    W110,
    /// An unknown key under `md`, or an unknown name in `md.syntax`.
    W111,
    /// Host metadata given for a document that has its own frontmatter.
    W115,
    /// A line starting with `::` inside an HTML block.
    W116,
}

impl Code {
    /// The code as the catalog writes it, `F004`.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::F001 => "F001",
            Self::F003 => "F003",
            Self::F004 => "F004",
            Self::F005 => "F005",
            Self::F007 => "F007",
            Self::F011 => "F011",
            Self::F012 => "F012",
            Self::F015 => "F015",
            Self::F016 => "F016",
            Self::F017 => "F017",
            Self::F032 => "F032",
            Self::F033 => "F033",
            Self::F036 => "F036",
            Self::W101 => "W101",
            Self::W106 => "W106",
            Self::W107 => "W107",
            Self::W108 => "W108",
            Self::W110 => "W110",
            Self::W111 => "W111",
            Self::W115 => "W115",
            Self::W116 => "W116",
        }
    }

    /// `Error` for `F` codes, `Warning` for `W` codes.
    #[must_use]
    pub fn severity(self) -> Severity {
        if self.as_str().starts_with('F') {
            Severity::Error
        } else {
            Severity::Warning
        }
    }
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for Code {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

/// How serious a diagnostic is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// An `F` code: strict mode fails (spec §10.1).
    Error,
    /// A `W` code.
    Warning,
}

/// The stage that found a diagnostic (spec §11).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    /// Stage 1, parsing.
    Parse,
}

/// A place in a document: 1-based line, 1-based column in code points.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct Location {
    /// The line.
    pub line: usize,
    /// The column, in code points.
    pub column: usize,
}

impl Location {
    /// The place at `line` and `column`.
    #[must_use]
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

impl From<crate::tree::Point> for Location {
    fn from(point: crate::tree::Point) -> Self {
        Self::new(point.line, point.column)
    }
}

/// Where a diagnostic, or a place related to it, is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Range {
    /// Where it starts.
    pub start: Location,
    /// Where it ends, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<Location>,
}

impl From<Location> for Range {
    fn from(start: Location) -> Self {
        Self { start, end: None }
    }
}

/// A place related to a diagnostic, such as the opener of an unclosed container.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Related {
    /// The document it is in, when it isn't the processed one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// Where it is.
    pub range: Range,
    /// What it is.
    pub message: String,
}

/// A diagnostic (spec §10.5).
///
/// ```
/// use compmd::diagnostic::{Code, Diagnostic, Location};
///
/// let diagnostic = Diagnostic::new(Code::F005, Location::new(3, 1), "no container is open for `::: /card`");
/// assert_eq!(
///     serde_json::to_value(&diagnostic).unwrap()["severity"],
///     "error"
/// );
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    /// The catalog code.
    pub code: Code,
    /// What is wrong, for people; not normative.
    pub message: String,
    /// The document it is in, when it isn't the processed one.
    pub file: Option<String>,
    /// Where it is.
    pub range: Range,
    /// Related places, innermost first.
    pub related: Vec<Related>,
    /// The stage that found it.
    pub stage: Stage,
}

impl Diagnostic {
    /// A parse diagnostic at `at` in the processed document.
    #[must_use]
    pub fn new(code: Code, at: Location, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            file: None,
            range: at.into(),
            related: Vec::new(),
            stage: Stage::Parse,
        }
    }

    /// The same diagnostic with a related place in the same document.
    #[must_use]
    pub fn with_related(mut self, at: Location, message: impl Into<String>) -> Self {
        self.related.push(Related {
            file: None,
            range: at.into(),
            message: message.into(),
        });
        self
    }

    /// The severity its code gives it.
    #[must_use]
    pub fn severity(&self) -> Severity {
        self.code.severity()
    }
}

impl Serialize for Diagnostic {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut out = serializer.serialize_struct("Diagnostic", 7)?;
        out.serialize_field("code", &self.code)?;
        out.serialize_field("severity", &self.severity())?;
        out.serialize_field("message", &self.message)?;
        if let Some(file) = &self.file {
            out.serialize_field("file", file)?;
        }
        out.serialize_field("range", &self.range)?;
        if !self.related.is_empty() {
            out.serialize_field("related", &self.related)?;
        }
        out.serialize_field("stage", &self.stage)?;
        out.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serializes_in_the_machine_form() {
        let diagnostic =
            Diagnostic::new(Code::F004, Location::new(1, 1), "`::: card` isn't closed")
                .with_related(Location::new(3, 1), "closed here");
        assert_eq!(
            serde_json::to_value(&diagnostic).unwrap(),
            json!({
                "code": "F004",
                "severity": "error",
                "message": "`::: card` isn't closed",
                "range": { "start": { "line": 1, "column": 1 } },
                "related": [{ "range": { "start": { "line": 3, "column": 1 } }, "message": "closed here" }],
                "stage": "parse",
            })
        );
    }

    #[test]
    fn names_a_file_that_is_not_the_processed_document() {
        let diagnostic = Diagnostic {
            file: Some("ui.comp.md".into()),
            range: Range {
                start: Location::new(7, 24),
                end: Some(Location::new(7, 32)),
            },
            ..Diagnostic::new(Code::F011, Location::new(1, 1), "")
        };
        let json = serde_json::to_value(&diagnostic).unwrap();
        assert_eq!(json["file"], "ui.comp.md");
        assert_eq!(json["range"]["end"], json!({ "line": 7, "column": 32 }));
    }

    #[test]
    fn warnings_are_w_codes() {
        assert_eq!(Code::W108.severity(), Severity::Warning);
        assert_eq!(Code::F016.to_string(), "F016");
    }
}
