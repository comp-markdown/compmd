//! An expression evaluated in the processed document's scope, as a developer's console asks: what `md.doc`,
//! `md.data`, `md.input` and the `md.` functions give for this render, without changing the document.

use super::{Expanded, Processor};
use crate::cel::{self, Program, Values};
use crate::diagnostic::{Code, Diagnostic, Location, Stage};
use crate::md::Signatures;
use crate::validate::scope::Scope;

impl Processor<Expanded> {
    /// Evaluates `expression`, a CEL expression of the profile without its parentheses (spec §8.2), in the scope the
    /// processed document's body sees at its top level: `md.doc`, `md.data`, `md.input` with the inputs' values this
    /// render used, `md.version`, `md.lang` and `md.tz`, and the `md.` functions, at this render's time. It is checked
    /// against the document's types first, as an expression in the document is, and evaluated within the same work
    /// budget. Nothing in the processor changes.
    ///
    /// # Errors
    ///
    /// F024, with the line and column in `expression` where it is: a syntax error, a type error such as an unknown
    /// name, field or function (with the nearest known one), or an evaluation error, the budget spent among them.
    ///
    /// ```
    /// use compmd::context::RenderContext;
    /// use compmd::{Ambient, ParseOptions, Processor};
    /// use std::rc::Rc;
    ///
    /// let text = "---\nteam: [Ana, Kenji]\n---\n\nThe team.\n";
    /// let context = RenderContext { now: 0, tz: "UTC".into(), time: Rc::new(()), inputs: Default::default() };
    /// let processor = Processor::new("a.comp.md", text)
    ///     .parse(&ParseOptions::default())
    ///     .resolve((), (), &Ambient::default())
    ///     .validate(&[])
    ///     .load(context, (), ())
    ///     .expand(&Default::default());
    /// assert_eq!(processor.evaluate("md.doc.team.size()").unwrap(), serde_json::json!(2));
    /// let error = processor.evaluate("md.doc.tem").unwrap_err();
    /// assert_eq!((error.code.as_str(), error.range.start.column), ("F024", 8));
    /// ```
    pub fn evaluate(&self, expression: &str) -> Result<serde_json::Value, Box<Diagnostic>> {
        let validated = &self.state.validated;
        let resolved = &validated.resolved;
        let main = resolved.main;
        let settings = &resolved.document(main).settings;
        let context = &self.state.render.context;
        let failed = |error: cel::Error| f024(expression, &error);
        let program = Program::parse(expression).map_err(failed)?;
        program
            .check(&Scope::body(resolved, main).cel(), &Signatures)
            .map_err(failed)?;
        let md = context
            .md(settings)
            .data(self.state.data.data(main))
            .input(&self.state.data.inputs);
        let values = Values::default().with("md", md.into());
        let value = program
            .evaluate(&values, &context.functions(settings))
            .map_err(failed)?;
        Ok(value.to_json())
    }
}

/// The F024 of `error` in `expression`, at its line and column there; parse and type errors are found as validating
/// finds a document's, evaluation errors as evaluating does.
fn f024(expression: &str, error: &cel::Error) -> Box<Diagnostic> {
    let stage = match error {
        cel::Error::Evaluation { .. } | cel::Error::Budget { .. } => Stage::Evaluate,
        cel::Error::Syntax { .. } | cel::Error::Type { .. } => Stage::Validate,
    };
    Box::new(
        Diagnostic::new(Code::F024, place(expression, error.at()), error.message()).in_stage(stage),
    )
}

/// The line and column, 1-based, of the code point `at` of `text`.
fn place(text: &str, at: usize) -> Location {
    let before: String = text.chars().take(at).collect();
    let line = before.matches('\n').count() + 1;
    let column = before
        .rsplit('\n')
        .next()
        .map_or(0, |last| last.chars().count())
        + 1;
    Location::new(line, column)
}

#[cfg(test)]
mod tests {
    use super::place;

    #[test]
    fn places_a_code_point_by_its_line_and_column() {
        assert_eq!(place("md.doc", 3), crate::diagnostic::Location::new(1, 4));
        assert_eq!(
            place("[1,\n  2,\n  é + x]", 15),
            crate::diagnostic::Location::new(3, 7)
        );
    }
}
