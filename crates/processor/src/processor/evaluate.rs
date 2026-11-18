//! An expression evaluated in the processed document's scope, as a developer's console asks: what `md.doc`,
//! `md.data`, `md.input` and the `md.` functions give for this render, without changing the document.

use super::{Expanded, Processor};
use crate::cel::{self, Program, Values};
use crate::diagnostic::{Code, Diagnostic};
use crate::lines::LineStarts;
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
        // F024 at the error's line and column in the expression, found by the stage that finds it in a document.
        let failed = |error: cel::Error| {
            let at = LineStarts::new(expression).location(error.at());
            Box::new(Diagnostic::new(Code::F024, at, error.message()).in_stage(error.stage()))
        };
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
