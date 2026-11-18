//! An edit of the text an expanded processor holds, processed again only as far as it reaches (spec §10.5).

use super::examples::Examples;
use super::{EditContext, Expanded, Parsed, Processor, Validated};
use crate::diagnostic::Diagnostic;
use crate::expand;
use crate::parse::{self, Change, TextEdit};
use crate::tree::Move;
use std::rc::Rc;

impl Processor<Expanded> {
    /// The same processor after `edit` of its text: the trees and diagnostics a processor of the new text gives,
    /// with everything it was given before, the parse options, the host's files, packs and data, the ambient packs,
    /// the allowed origins, the render context and the expansion options.
    ///
    /// Parsing only parses again the blocks the edit touches and one block either side, and moves the blocks after
    /// them; an edit to the link and footnote definitions parses the whole text again. The later stages then only
    /// run on the top-level blocks whose tree changed, and move what the others gave. An edit to the frontmatter or to
    /// a component definition or signature fence runs them on the whole document again; an edit that changes which
    /// data imports may load loads again.
    ///
    /// ```
    /// use compmd::context::RenderContext;
    /// use compmd::expand::Options;
    /// use compmd::{Ambient, ParseOptions, Processor, TextEdit};
    /// use std::collections::HashMap;
    /// use std::rc::Rc;
    ///
    /// let files: HashMap<String, String> = HashMap::new();
    /// let context = RenderContext { now: 0, tz: "UTC".into(), time: Rc::new(()), inputs: Default::default() };
    /// let processor = Processor::new("a.comp.md", "Hello world\n")
    ///     .parse(&ParseOptions::default())
    ///     .resolve(files, (), &Ambient::default())
    ///     .validate(&[])
    ///     .load(context, (), ())
    ///     .expand(&Options::default());
    ///
    /// // Select `world` and type `there`.
    /// let processor = processor.edit(&TextEdit { start: 6, end: 11, text: "there".into() });
    /// let json = serde_json::to_value(processor.expanded()).unwrap();
    /// assert_eq!(json["children"][0]["children"][0]["value"], "Hello there");
    /// ```
    #[must_use]
    pub fn edit(self, edit: &TextEdit) -> Self {
        let Expanded {
            validated,
            render,
            mut data,
            options,
            result,
            ..
        } = self.state;
        let Validated {
            store,
            mut resolved,
            mut checks,
            edit_context: context,
            allowed_origins,
            ..
        } = validated;
        let before = result.root.children.len();
        let (text, mut lines) = resolved.take_text();
        let (text, document, change) =
            parse::reparse(context.parsed, &text, &mut lines, edit, &context.options);
        let Change::Blocks(spliced) = change else {
            drop(resolved);
            let mut diagnostics = Vec::new();
            let parsed = Parsed::new(
                context.path,
                text,
                document,
                context.options,
                &mut diagnostics,
            );
            let mut processor = Processor {
                diagnostics,
                state: parsed,
            }
            .resolve_in(store, context.ambient, context.policy, context.parses)
            .validate(&allowed_origins)
            .load_with(render)
            .expand(&options);
            processor.state.replaced.nodes = 0..before;
            return processor;
        };
        let report = context.ambient.report;
        resolved.splice((text, lines), Rc::clone(&document.root), &spliced, report);
        let reload = checks.splice(&resolved, &spliced, &allowed_origins);
        let mut diagnostics = document.diagnostics.clone();
        diagnostics.extend(resolved.diagnostics().cloned());
        diagnostics.extend(checks.diagnostics().cloned());
        let validated = with_examples(
            Validated {
                store,
                resolved,
                checks,
                examples: Examples::default(),
                kept: 0,
                edit_context: EditContext {
                    parsed: document,
                    ..context
                },
                allowed_origins,
            },
            &mut diagnostics,
        );
        // A block the edit didn't touch may hold an example whose number or place it changed: a document with
        // examples is loaded and expanded whole, as after a reload.
        if reload || !validated.examples.is_empty() {
            let mut processor = Processor {
                diagnostics,
                state: validated,
            }
            .load_with(render)
            .expand(&options);
            processor.state.replaced.nodes = 0..before;
            return processor;
        }
        diagnostics.extend_from_slice(&data.diagnostics);
        // The document has no examples now: none of those it had stays.
        data.examples.clear();
        let result = expand::splice(
            result,
            &validated.resolved,
            &validated.checks,
            &data,
            &render.context,
            &options,
            &spliced,
        );
        diagnostics.extend_from_slice(&result.diagnostics);
        let replaced = result.replaced.clone().unwrap_or(expand::Replaced {
            nodes: 0..before,
            inserted: result.root.children.len(),
            by: Move::default(),
        });
        Processor {
            diagnostics,
            state: Expanded {
                validated,
                render,
                data,
                options,
                result,
                replaced,
            },
        }
    }
}

/// `validated` with its document's examples (spec §7.8), each processed again: an example is small, and its text may
/// have changed with its block. Their diagnostics go to `diagnostics`, which stages 1 to 3 then end with.
fn with_examples(mut validated: Validated, diagnostics: &mut Vec<Diagnostic>) -> Validated {
    validated.examples = Examples::of(
        &validated.resolved,
        &validated.store,
        &validated.edit_context,
        &validated.allowed_origins,
        diagnostics,
    );
    validated.kept = diagnostics.len();
    validated
}
