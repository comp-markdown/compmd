//! The processor's diagnostics as LSP's ([the language server], *The core*): each with its code, its severity, its
//! range, its related places and a link to the spec's catalog. One in another document (an imported one, a pack's)
//! is shown at the top of this one, naming where it is.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#the-core-documents-diagnostics-settings

use compmd::diagnostic::{Diagnostic, Severity};
use lsp_types::{
    CodeDescription, Diagnostic as LspDiagnostic, DiagnosticRelatedInformation, DiagnosticSeverity,
    Location, NumberOrString, Position, Range, Uri,
};

use crate::encoding::Encoding;

/// Where the spec lists every diagnostic code (§10.2).
const CATALOG: &str = "https://compmd.dev/spec/#102-catalog";

/// What every diagnostic says it comes from.
pub(crate) const SOURCE: &str = "compmd";

/// The top of a document, where what is about another one is shown.
const TOP: Range = Range {
    start: Position {
        line: 0,
        character: 0,
    },
    end: Position {
        line: 0,
        character: 0,
    },
};

/// `diagnostic`, of the document `uri` whose text is `text`, as LSP's.
pub(crate) fn lsp_diagnostic(
    diagnostic: &Diagnostic,
    uri: &Uri,
    text: &str,
    encoding: Encoding,
) -> LspDiagnostic {
    let (range, message) = match &diagnostic.file {
        None => (
            encoding.range(text, &diagnostic.range),
            diagnostic.message.clone(),
        ),
        Some(file) => {
            let at = diagnostic.range.start;
            (
                TOP,
                format!("{file}:{}:{}: {}", at.line, at.column, diagnostic.message),
            )
        }
    };
    let related: Vec<DiagnosticRelatedInformation> = diagnostic
        .related
        .iter()
        .filter(|related| related.file.is_none())
        .map(|related| DiagnosticRelatedInformation {
            location: Location {
                uri: uri.clone(),
                range: encoding.range(text, &related.range),
            },
            message: related.message.clone(),
        })
        .collect();
    LspDiagnostic {
        range,
        severity: Some(match diagnostic.severity() {
            Severity::Error => DiagnosticSeverity::ERROR,
            Severity::Warning => DiagnosticSeverity::WARNING,
        }),
        code: Some(NumberOrString::String(diagnostic.code.to_string())),
        code_description: CATALOG.parse().ok().map(|href| CodeDescription { href }),
        source: Some(SOURCE.to_owned()),
        message,
        related_information: (!related.is_empty()).then_some(related),
        ..LspDiagnostic::default()
    }
}

/// What stops a document being processed at all (a configuration that's wrong, a pack that isn't one), as an error
/// at its top: errors are diagnostics, never crashes.
pub(crate) fn project_diagnostic(message: String) -> LspDiagnostic {
    LspDiagnostic {
        range: TOP,
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some(SOURCE.to_owned()),
        message,
        ..LspDiagnostic::default()
    }
}

#[cfg(test)]
mod tests {
    use compmd::{Ambient, ParseOptions, Processor};

    use super::*;

    fn uri() -> Uri {
        "file:///site/a.comp.md".parse().expect("a URI")
    }

    /// The diagnostics of `text`, parsed and resolved, as LSP's.
    fn diagnostics(text: &str) -> Vec<LspDiagnostic> {
        let resolved = Processor::new("a.comp.md", text)
            .parse(&ParseOptions::default())
            .resolve((), (), &Ambient::default());
        resolved
            .diagnostics()
            .iter()
            .map(|diagnostic| lsp_diagnostic(diagnostic, &uri(), text, Encoding::Utf16))
            .collect()
    }

    #[test]
    fn a_diagnostic_has_its_code_severity_range_and_the_catalogs_link() {
        let [unknown] = diagnostics("# A\n\n:: nowhere[Here]\n")
            .try_into()
            .expect("one");
        assert_eq!(unknown.code, Some(NumberOrString::String("F002".into())));
        assert_eq!(unknown.severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(
            unknown.range.start,
            Position {
                line: 2,
                character: 3
            }
        );
        assert_eq!(unknown.source.as_deref(), Some("compmd"));
        assert_eq!(
            unknown
                .code_description
                .map(|description| description.href.to_string())
                .as_deref(),
            Some(CATALOG)
        );
    }

    #[test]
    fn a_warning_is_a_warning_and_related_places_come_along() {
        // A container never closed: F004, related to where it opens.
        let found = diagnostics("::: card\nText.\n");
        let twice = found
            .iter()
            .find(|diagnostic| diagnostic.related_information.is_some())
            .expect("one with a related place");
        assert_eq!(twice.related_information.as_ref().map(Vec::len), Some(1));
        let leaf = diagnostics(":: x\n");
        assert!(
            leaf.iter()
                .any(|diagnostic| diagnostic.severity == Some(DiagnosticSeverity::WARNING)),
            "{leaf:?}"
        );
    }

    #[test]
    fn one_in_another_document_is_at_the_top_naming_where() {
        let mut diagnostic = compmd::diagnostic::Diagnostic::new(
            compmd::diagnostic::Code::F002,
            compmd::diagnostic::Location::new(3, 4),
            "unknown",
        );
        diagnostic.file = Some("@kit/x@1.0.0/index.comp.md".into());
        let converted = lsp_diagnostic(&diagnostic, &uri(), "# A\n", Encoding::Utf16);
        assert_eq!(converted.range, TOP);
        assert_eq!(converted.message, "@kit/x@1.0.0/index.comp.md:3:4: unknown");
        let project = project_diagnostic("compmd.config.json:\n- fetch: `host` or `direct`".into());
        assert_eq!(
            (project.range, project.severity),
            (TOP, Some(DiagnosticSeverity::ERROR))
        );
    }
}
