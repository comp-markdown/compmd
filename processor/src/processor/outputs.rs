//! What a processor gives once it expanded the document: its trees, its frontmatter and components, the hints at a
//! caret, and what a render needs or treated as pending.

use super::{Expanded, Processor, Splice, back};
use crate::context::RenderContext;
use crate::expanded;
use crate::host::{CacheKey, Data, Fetch, Needed, Pending};
use crate::resolve::HostComponent;
use crate::{load, tree};

impl Processor<Expanded> {
    /// What the processed document's frontmatter gives the render: its own fields, its settings, each data import
    /// with what was loaded, and its inputs.
    ///
    /// ```
    /// use compmd::context::RenderContext;
    /// use compmd::expand::Options;
    /// use compmd::{Ambient, ParseOptions, Processor};
    /// use std::collections::HashMap;
    /// use std::rc::Rc;
    ///
    /// let text = "---\ntitle: Team\nmd:\n  data:\n    - from: ./team.csv\n      as: team\n---\n";
    /// let files = HashMap::from([("team.csv".to_string(), "name\nAna\n".to_string())]);
    /// let context = RenderContext { now: 0, tz: "UTC".into(), time: Rc::new(()), inputs: Default::default() };
    /// let processor = Processor::new("a.comp.md", text)
    ///     .parse(&ParseOptions::default())
    ///     .resolve(files, (), &Ambient::default())
    ///     .validate(&[])
    ///     .load(context, (), ())
    ///     .expand(&Options::default());
    /// let frontmatter = processor.frontmatter();
    /// assert_eq!(frontmatter.doc["title"], "Team");
    /// assert_eq!(frontmatter.data[0].value, serde_json::json!([{ "name": "Ana" }]));
    /// ```
    #[must_use]
    pub fn frontmatter(&self) -> crate::frontmatter::Frontmatter {
        let resolved = &self.state.validated.resolved;
        crate::frontmatter::of(resolved, resolved.main, &self.state.data)
    }

    /// The components the processed document can name, grouped by the file that defines them, each with where the
    /// document uses it, and where it defines it; and the names it uses that stand for none.
    ///
    /// ```
    /// use compmd::context::RenderContext;
    /// use compmd::expand::Options;
    /// use compmd::{Ambient, ParseOptions, Processor};
    /// use std::collections::HashMap;
    /// use std::rc::Rc;
    ///
    /// let text = "```compmd component=note\n---\nA note.\n```\n\n:: note[]\n";
    /// let files: HashMap<String, String> = HashMap::new();
    /// let context = RenderContext { now: 0, tz: "UTC".into(), time: Rc::new(()), inputs: Default::default() };
    /// let processor = Processor::new("a.comp.md", text)
    ///     .parse(&ParseOptions::default())
    ///     .resolve(files, (), &Ambient::default())
    ///     .validate(&[])
    ///     .load(context, (), ())
    ///     .expand(&Options::default());
    /// let components = processor.components();
    /// let here = &components.groups[0];
    /// assert_eq!((here.file.as_deref(), here.components[0].name.as_str()), (None, "note"));
    /// assert_eq!(here.components[0].uses[0].start.line, 6);
    /// ```
    #[must_use]
    pub fn components(&self) -> crate::components::Components {
        crate::components::of(&self.state.validated.resolved)
    }

    /// What is at code point `offset` of the processed document's text, and what it takes: a component's name, a
    /// param's key or value, a param list, a slot, with the diagnostics there and what may be written there. `None`
    /// where there is nothing to say, such as in plain prose. It is a lookup in what the processor keeps, so it can
    /// answer on every move of a caret.
    ///
    /// ```
    /// use compmd::context::RenderContext;
    /// use compmd::hints::Subject;
    /// use compmd::{Ambient, ParseOptions, Processor};
    /// use std::rc::Rc;
    ///
    /// let text = "::: box {border=subtle}\nText.\n::: /box\n";
    /// let context = RenderContext { now: 0, tz: "UTC".into(), time: Rc::new(()), inputs: Default::default() };
    /// let processor = Processor::new("a.comp.md", text)
    ///     .parse(&ParseOptions::default())
    ///     .resolve((), (), &Ambient::default())
    ///     .validate(&[])
    ///     .load(context, (), ())
    ///     .expand(&Default::default());
    /// // The caret in `subtle`.
    /// let hint = processor.hint_at(18).unwrap();
    /// let Some(Subject::Param { param, value, .. }) = &hint.subject else { panic!("a param") };
    /// assert_eq!((param.name.as_str(), value.as_deref()), ("border", Some("subtle")));
    /// let values: Vec<&str> = hint.completions.iter().map(|c| c.text.as_str()).collect();
    /// assert_eq!(values, ["none", "subtle", "strong"]);
    /// ```
    #[must_use]
    pub fn hint_at(&self, offset: usize) -> Option<crate::hints::PositionalHint> {
        crate::hints::at(&self.state.validated.resolved, &self.diagnostics, offset)
    }

    /// The processed document's tree (spec §12.1).
    #[must_use]
    pub fn tree(&self) -> &tree::Root {
        let resolved = &self.state.validated.resolved;
        &resolved.document(resolved.main).root
    }

    /// Whether processing failed: in strict mode ([`expand::Options::mode`](crate::expand::Options::mode)), any `F`
    /// diagnostic fails it (spec §10.1). The tree and the diagnostics are there either way.
    ///
    /// ```
    /// use compmd::context::RenderContext;
    /// use compmd::diagnostic::Mode;
    /// use compmd::expand::Options;
    /// use compmd::{Ambient, ParseOptions, Processor};
    /// use std::rc::Rc;
    ///
    /// let context = RenderContext { now: 0, tz: "UTC".into(), time: Rc::new(()), inputs: Default::default() };
    /// let strict = Options { mode: Mode::Strict, ..Options::default() };
    /// let processor = Processor::new("a.comp.md", "Hello {{nobody}}.\n")
    ///     .parse(&ParseOptions::default())
    ///     .resolve((), (), &Ambient::default())
    ///     .validate(&[])
    ///     .load(context, (), ())
    ///     .expand(&strict);
    /// assert!(processor.failed());
    /// assert!(!processor.expanded().children.is_empty());
    /// ```
    #[must_use]
    pub fn failed(&self) -> bool {
        self.state.options.mode.fails(&self.diagnostics)
    }

    /// The cache key of each of the processed document's data imports, for the input values this render used: the
    /// final URL or path a host keys its cached copy by (spec §8.6).
    #[must_use]
    pub fn cache_keys(&self) -> Vec<CacheKey> {
        let validated = &self.state.validated;
        load::cache_keys(
            &validated.resolved,
            &validated.checks,
            &self.state.data.inputs,
        )
    }

    /// What the render treated as pending (spec §6.4, §8.6): the packs, remote URLs and `host:` sources it needed that
    /// the host said it was still getting, whose uses render their fallbacks marked as pending and whose data is its
    /// `fallback` or `null`, without F026.
    ///
    /// ```
    /// use compmd::context::RenderContext;
    /// use compmd::host::WithPending;
    /// use compmd::{Ambient, ParseOptions, Processor};
    /// use std::collections::HashSet;
    /// use std::rc::Rc;
    ///
    /// let text = "---\nmd:\n  import:\n    - from: \"@kit/charts@1\"\n      names: [chart]\n---\n\n:: chart[Sales by month]\n";
    /// let packs = WithPending { held: (), pending: HashSet::from(["@kit/charts".to_string()]) };
    /// let context = RenderContext { now: 0, tz: "UTC".into(), time: Rc::new(()), inputs: Default::default() };
    /// let processor = Processor::new("a.comp.md", text)
    ///     .parse(&ParseOptions::default())
    ///     .resolve((), packs, &Ambient::default())
    ///     .validate(&[])
    ///     .load(context, (), ())
    ///     .expand(&Default::default());
    /// assert!(processor.diagnostics().is_empty());
    /// assert_eq!(processor.pending().packs, ["@kit/charts"]);
    /// let json = serde_json::to_value(processor.expanded()).unwrap();
    /// assert_eq!(json["children"][0]["type"], "componentPending");
    /// ```
    #[must_use]
    pub fn pending(&self) -> Pending {
        Pending {
            packs: self.state.validated.resolved.pending.clone(),
            ..self.state.data.pending.clone()
        }
    }

    /// The expanded tree (spec §12.3).
    #[must_use]
    pub fn expanded(&self) -> &expanded::Root {
        &self.state.result.root
    }

    /// What the last edit changed in the expanded tree; after the first expansion, all of it.
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
    /// let processor = Processor::new("a.comp.md", "One.\n\nTwo.\n\nThree.\n")
    ///     .parse(&ParseOptions::default())
    ///     .resolve(files, (), &Ambient::default())
    ///     .validate(&[])
    ///     .load(context, (), ())
    ///     .expand(&Options::default());
    /// assert_eq!(processor.splice().nodes.len(), 3);
    ///
    /// // Type a new line into `Two.`: the paragraph is replaced, and `Three.` moves a line down.
    /// let processor = processor.edit(&TextEdit { start: 9, end: 9, text: "\n2".into() });
    /// let splice = processor.splice();
    /// assert_eq!((splice.from, splice.to, splice.nodes.len()), (1, 2, 1));
    /// assert_eq!((splice.lines, splice.offset), (1, 2));
    /// ```
    #[must_use]
    pub fn splice(&self) -> Splice {
        let replaced = &self.state.replaced;
        let from = replaced.nodes.start;
        Splice {
            from,
            to: replaced.nodes.end,
            nodes: self.state.result.root.children[from..from + replaced.inserted].to_vec(),
            lines: replaced.by.lines,
            offset: replaced.by.offset,
        }
    }

    /// Loads and expands again for a render with `context`, `fetch` and `data`, with the same options, without
    /// running stages 1 to 3 again.
    #[must_use]
    pub fn reload(
        self,
        context: RenderContext,
        fetch: impl Fetch + 'static,
        data: impl Data + 'static,
    ) -> Processor<Expanded> {
        let options = self.state.options.clone();
        let before = self.state.result.root.children.len();
        let mut processor = self
            .then(|expanded, diagnostics| back(expanded.validated, diagnostics))
            .load(context, fetch, data)
            .expand(&options);
        processor.state.replaced.nodes = 0..before;
        processor
    }

    /// The host-language components the expanded tree can name, with their code, by name: what a renderer that
    /// runs them needs, since the tree names a component but doesn't hold its code (spec §7.4, §12.3). Only those
    /// one of the processor's bindings runs (others render their fallback, W102); a name two documents give
    /// different components is the processed document's, or else the first loaded.
    ///
    /// ```
    /// use compmd::context::RenderContext;
    /// use compmd::{Ambient, Language, ParseOptions, Processor};
    /// use std::rc::Rc;
    ///
    /// let text = "```jsx component=hello runtime=react@19\nexport default () => <b>Hi</b>;\n```\n\n:: hello[Hi]\n";
    /// let context = RenderContext { now: 0, tz: "UTC".into(), time: Rc::new(()), inputs: Default::default() };
    /// let processor = Processor::new("a.comp.md", text)
    ///     .parse(&ParseOptions::default())
    ///     .resolve((), (), &Ambient::default())
    ///     .validate(&[])
    ///     .load(context, (), ())
    ///     .expand(&Default::default());
    /// let [hello] = processor.host_components().try_into().unwrap();
    /// assert_eq!((hello.name.as_str(), hello.language), ("hello", Language::Jsx));
    /// assert_eq!(hello.code, "export default () => <b>Hi</b>;");
    /// ```
    #[must_use]
    pub fn host_components(&self) -> Vec<HostComponent> {
        let validated = &self.state.validated;
        validated.resolved.host_components(validated.store.store())
    }

    /// The remote and `host:` sources a render needs with the context the processor keeps, and the packs it has no
    /// release of, as [`sources`](Processor#method.sources) gives them before loading. After an edit to the
    /// frontmatter, a host that fetches ahead fetches what it hasn't yet and calls
    /// [`reload`](Processor#method.reload-1); a pack it gets needs the document processed again with it.
    #[must_use]
    pub fn sources(&self) -> Needed {
        let validated = &self.state.validated;
        load::sources(
            &validated.resolved,
            &validated.checks,
            &self.state.render.context,
            &validated.allowed_origins,
        )
    }
}
