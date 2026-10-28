//! Diagnostics (spec §10) in the machine form of spec §10.5.

use crate::named::named;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use std::fmt;

named! {
    /// A diagnostic code from the catalog (spec §10.2): `F…` codes are errors, `W…` codes warnings.
    #[derive(PartialOrd, Ord, Hash)]
    pub enum Code {
        /// Unsupported major version.
        F001 = "F001",
        /// An unknown component name, or a signature fence with nothing to attach to.
        F002 = "F002",
        /// A closer names no open container while one is open.
        F003 = "F003",
        /// A container isn't closed before an enclosing closer or the end of the file.
        F004 = "F004",
        /// A closer while no container is open.
        F005 = "F005",
        /// An undeclared slot name.
        F006 = "F006",
        /// A named slot given twice in one container.
        F007 = "F007",
        /// An unknown param, or a breakpoint the param doesn't take.
        F008 = "F008",
        /// A param value that doesn't match its type or grammar.
        F009 = "F009",
        /// A required param is missing.
        F010 = "F010",
        /// A malformed param list.
        F011 = "F011",
        /// A continuation line that doesn't follow an opener, a leaf or a continuation line.
        F012 = "F012",
        /// A name declared twice, a keyword or `md` declared, or a name expressions can't use.
        F013 = "F013",
        /// An unknown interpolation path.
        F014 = "F014",
        /// A slot line outside a container.
        F015 = "F015",
        /// A malformed component line.
        F016 = "F016",
        /// A param given twice, or `id` together with `#name`.
        F017 = "F017",
        /// A body given to a component that declares no slots.
        F018 = "F018",
        /// A recursive component definition.
        F019 = "F019",
        /// An import cycle.
        F020 = "F020",
        /// An imported name its source doesn't export.
        F021 = "F021",
        /// `export.pack` or `export.version` disagrees with the registry manifest.
        F022 = "F022",
        /// A name two wildcard imports, or a wildcard import and `@compmd/core`, provide is used.
        F023 = "F023",
        /// An expression's parse, type or evaluation error.
        F024 = "F024",
        /// A data or import source missing, unreachable or not allowed.
        F026 = "F026",
        /// A remote data origin the processor's configuration doesn't allow.
        F028 = "F028",
        /// An exported host-language component without a signature fence.
        F029 = "F029",
        /// A bound input that isn't declared, or has another type.
        F030 = "F030",
        /// A `from` that interpolates what it may not.
        F031 = "F031",
        /// A closer names a container opened outside its list item or block quote.
        F032 = "F032",
        /// Frontmatter that isn't YAML 1.2 core, or uses tags, anchors, aliases or repeated keys.
        F033 = "F033",
        /// A pack that doesn't match its import's `integrity`.
        F035 = "F035",
        /// A param list on a signature fence.
        F036 = "F036",
        /// A leaf without fallback text.
        W101 = "W101",
        /// A definition language or runtime this processor doesn't support.
        W102 = "W102",
        /// An `each` whose default slot is a single list item, with `join` unset.
        W103 = "W103",
        /// A local host-language component whose params can't be typed.
        W105 = "W105",
        /// `{{` followed by something that isn't a path.
        W106 = "W106",
        /// An inline component followed by a space and a param list.
        W107 = "W107",
        /// A container opener or leaf directly after a paragraph line.
        W108 = "W108",
        /// A later minor version than the processor implements.
        W110 = "W110",
        /// An unknown key under `md`, or an unknown name in `md.syntax`.
        W111 = "W111",
        /// A local definition or explicit import shadows a `@compmd/core` component.
        W112 = "W112",
        /// A name from an ambient pack is used.
        W113 = "W113",
        /// Host metadata given for a document that has its own frontmatter.
        W115 = "W115",
        /// A line starting with `::` inside an HTML block.
        W116 = "W116",
        /// An exported host-language component without `runtime`.
        W117 = "W117",
        /// A param for a box given to a `display=contents` component.
        W120 = "W120",
    }
}

impl Code {
    /// The code as the catalog writes it, `F004`.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.name()
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
    /// Stage 2, resolving names.
    Resolve,
    /// Stage 3, validating.
    Validate,
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

    /// The same diagnostic, found by `stage`.
    #[must_use]
    pub fn in_stage(self, stage: Stage) -> Self {
        Self { stage, ..self }
    }

    /// The same diagnostic, in the document named `file`; `None` is the processed document.
    #[must_use]
    pub fn in_file(self, file: Option<String>) -> Self {
        Self { file, ..self }
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
