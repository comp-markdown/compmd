//! What a processor gives once it expanded the document: its trees, its frontmatter and components, the hints at a
//! caret, and what a render needs or treated as pending.

use super::{Expanded, Processor, Splice, back};
use crate::context::RenderContext;
use crate::diagnostic::Location;
use crate::expanded;
use crate::host::{CacheKey, Data, Fetch, Needed, Pending};
use crate::resolve::{HostComponent, Origin};
use crate::{load, tree};
use compmd_pack::ReleaseId;

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

    /// The processed document's code blocks that a code renderer renders (spec §7.7), in written order, each with its
    /// language, its renderer and where its text is: what an editor needs to hand the requests inside a block to the
    /// writer's language server for that language.
    ///
    /// ```
    /// use compmd::context::RenderContext;
    /// use compmd::{Ambient, ParseOptions, Processor};
    /// use std::rc::Rc;
    ///
    /// let text = "```compmd component=diagram\nrenders: mermaid\n---\n{{code}}\n```\n\n```mermaid\ngraph LR; A --> B\n```\n";
    /// let context = RenderContext { now: 0, tz: "UTC".into(), time: Rc::new(()), inputs: Default::default() };
    /// let processor = Processor::new("a.comp.md", text)
    ///     .parse(&ParseOptions::default())
    ///     .resolve((), (), &Ambient::default())
    ///     .validate(&[])
    ///     .load(context, (), ())
    ///     .expand(&Default::default());
    /// let [block] = processor.rendered_blocks().try_into().unwrap();
    /// assert_eq!((block.language.as_str(), block.component.as_str()), ("mermaid", "diagram"));
    /// let range = block.range;
    /// assert_eq!(&text[range.start.offset..range.end.offset], "graph LR; A --> B");
    /// ```
    #[must_use]
    pub fn rendered_blocks(&self) -> Vec<crate::rendered_blocks::RenderedBlock> {
        crate::rendered_blocks::of(&self.state.validated.resolved)
    }

    /// What is at code point `offset` of the processed document's text, and what it takes: a component's name, a
    /// param's key or value, a param list, a slot, with the diagnostics there and what may be written there. `None`
    /// where there is nothing to say, such as in plain prose, and in a plain Markdown document
    /// ([`Self::is_plain_markdown`]). It is a lookup in what the processor keeps, so it can answer on every move of a
    /// caret.
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
        if self.is_plain_markdown() {
            return None;
        }
        crate::hints::at(&self.state.validated.resolved, &self.diagnostics, offset)
    }

    /// What is being written at code point `offset`, for an editor's signature help: the call of an expression the
    /// offset is in, with the function and the argument it is at, or the param list of a component, with every param
    /// it takes and the one the offset is on. A lookup, as [`Self::hint_at`] is.
    ///
    /// ```
    /// use compmd::context::RenderContext;
    /// use compmd::hints::SignatureHelp;
    /// use compmd::{Ambient, ParseOptions, Processor};
    /// use std::rc::Rc;
    ///
    /// let text = "::: box {padding=(md.number(1, ))}\n::: /box\n";
    /// let context = RenderContext { now: 0, tz: "UTC".into(), time: Rc::new(()), inputs: Default::default() };
    /// let processor = Processor::new("a.comp.md", text)
    ///     .parse(&ParseOptions::default())
    ///     .resolve((), (), &Ambient::default())
    ///     .validate(&[])
    ///     .load(context, (), ())
    ///     .expand(&Default::default());
    /// // After `md.number(1, `.
    /// let Some(SignatureHelp::Call { function, argument }) = processor.signature_at(31) else { panic!("a call") };
    /// assert_eq!((function.name.as_str(), argument), ("md.number", 1));
    /// ```
    #[must_use]
    pub fn signature_at(&self, offset: usize) -> Option<crate::hints::SignatureHelp> {
        if self.is_plain_markdown() {
            return None;
        }
        crate::hints::signature_at(&self.state.validated.resolved, offset)
    }

    /// Where what code point `offset` names is defined, for an editor's "go to definition": a component's name, its
    /// definition fence or the file or pack document that exports it; an import's `from`, what it loads; a path
    /// `md.data.<name>`, its data import. A lookup, as [`Self::hint_at`] is.
    ///
    /// ```
    /// use compmd::context::RenderContext;
    /// use compmd::{Ambient, ParseOptions, Processor};
    /// use std::rc::Rc;
    ///
    /// let text = "```compmd component=note\n---\nA note.\n```\n\n:: note[N]\n";
    /// let context = RenderContext { now: 0, tz: "UTC".into(), time: Rc::new(()), inputs: Default::default() };
    /// let processor = Processor::new("a.comp.md", text)
    ///     .parse(&ParseOptions::default())
    ///     .resolve((), (), &Ambient::default())
    ///     .validate(&[])
    ///     .load(context, (), ())
    ///     .expand(&Default::default());
    /// // On `note` in `:: note[N]`.
    /// let defined = processor.definition_at(48).unwrap();
    /// assert_eq!((defined.file, defined.range.map(|range| range.start.line)), (None, Some(1)));
    /// ```
    #[must_use]
    pub fn definition_at(&self, offset: usize) -> Option<crate::hints::Defined> {
        if self.is_plain_markdown() {
            return None;
        }
        crate::hints::definition_at(&self.state.validated.resolved, offset)
    }

    /// The document's syntax tokens ([`crate::tokens`]): where each span of CompMD's syntax is and what it is, in the
    /// order they start, a name standing for no component marked `unknown`. A lookup, as [`Self::hint_at`] is: each
    /// line is read again, so a line being typed has its tokens too. None in a plain Markdown document.
    ///
    /// ```
    /// use compmd::context::RenderContext;
    /// use compmd::tokens::TokenKind;
    /// use compmd::{Ambient, ParseOptions, Processor};
    /// use std::rc::Rc;
    ///
    /// let context = RenderContext { now: 0, tz: "UTC".into(), time: Rc::new(()), inputs: Default::default() };
    /// let processor = Processor::new("a.comp.md", ":: carx[A card] {title=\"Hi\"}\n")
    ///     .parse(&ParseOptions::default())
    ///     .resolve((), (), &Ambient::default())
    ///     .validate(&[])
    ///     .load(context, (), ())
    ///     .expand(&Default::default());
    /// let tokens = processor.tokens();
    /// let kinds: Vec<TokenKind> = tokens.iter().map(|token| token.kind).collect();
    /// assert_eq!(kinds, [TokenKind::ComponentMarker, TokenKind::ComponentName, TokenKind::ParamKey, TokenKind::ParamString]);
    /// assert!(tokens[1].unknown, "`carx` stands for no component");
    /// ```
    #[must_use]
    pub fn tokens(&self) -> Vec<crate::tokens::SyntaxToken> {
        if self.is_plain_markdown() {
            return Vec::new();
        }
        let resolved = &self.state.validated.resolved;
        crate::tokens::of(resolved.document(resolved.main))
    }

    /// Every place the processed document reads the metadata path `path` (`md.doc.due_date`, `md.input.week`,
    /// `md.data.sales`): each read of its key, in every expression the document writes (a param's `(…)` on a component
    /// line, an inline component or a definition fence, and an interpolation's `{{…}}`, in text and in quoted params),
    /// by dot, in `has(…)`, or by a string literal index; and what may read it and can't be renamed, an index computed
    /// when it runs (`md.doc[name]`). Text that only looks like one, in a code block, in prose or inside a CEL string,
    /// isn't one. None in a plain Markdown document. A lookup, as [`Self::hint_at`] is.
    ///
    /// # Errors
    ///
    /// [`PathError::NotAPath`](crate::path_references::PathError::NotAPath) for a path that isn't `md` and two names
    /// or more.
    ///
    /// ```
    /// use compmd::context::RenderContext;
    /// use compmd::{Ambient, ParseOptions, Processor};
    /// use std::rc::Rc;
    ///
    /// let text = "---\ndue_date: \"1 December\"\n---\n\nDue {{md.doc.due_date}}.\n\n:: badge[Due] {show=(has(md.doc.due_date))}\n";
    /// let context = RenderContext { now: 0, tz: "UTC".into(), time: Rc::new(()), inputs: Default::default() };
    /// let processor = Processor::new("a.comp.md", text)
    ///     .parse(&ParseOptions::default())
    ///     .resolve((), (), &Ambient::default())
    ///     .validate(&[])
    ///     .load(context, (), ())
    ///     .expand(&Default::default());
    /// let found = processor.path_references("md.doc.due_date").unwrap();
    /// let chars: Vec<char> = text.chars().collect();
    /// let written: Vec<String> = found.reads.iter().map(|read| chars[read.start..read.end].iter().collect()).collect();
    /// assert_eq!(written, ["due_date", "due_date"]);
    /// ```
    pub fn path_references(
        &self,
        path: &str,
    ) -> Result<crate::path_references::PathReferences, crate::path_references::PathError> {
        let path = crate::path_references::MetadataPath::new(path)?;
        if self.is_plain_markdown() {
            return Ok(crate::path_references::PathReferences::default());
        }
        let resolved = &self.state.validated.resolved;
        Ok(crate::path_references::find_references(
            resolved.document(resolved.main),
            &path,
        ))
    }

    /// The edits that rename the key of the metadata path `path` to `key` wherever the processed document reads it,
    /// as [`Self::path_references`] finds them: each over the key's name only, so `md.doc.due_date` becomes
    /// `md.doc.due` and nothing else changes; and what may read it that they can't rename, for the caller to report.
    /// The frontmatter's own key, if it has one, isn't among them: it is the host's, or the writer's, to rename.
    ///
    /// # Errors
    ///
    /// [`PathError::NotAPath`](crate::path_references::PathError::NotAPath) for `path`, as
    /// [`Self::path_references`]; [`PathError::NotAKey`](crate::path_references::PathError::NotAKey) for a `key` an
    /// expression can't select by dot.
    ///
    /// ```
    /// use compmd::context::RenderContext;
    /// use compmd::{Ambient, ParseOptions, Processor};
    /// use std::rc::Rc;
    ///
    /// let text = "Due {{md.doc.due_date}}.\n\n:: badge[Due] {title=(md.doc['due_date'])}\n";
    /// let context = RenderContext { now: 0, tz: "UTC".into(), time: Rc::new(()), inputs: Default::default() };
    /// let processor = Processor::new("a.comp.md", text)
    ///     .parse(&ParseOptions::default())
    ///     .resolve((), (), &Ambient::default())
    ///     .validate(&[])
    ///     .load(context, (), ())
    ///     .expand(&Default::default());
    /// let renamed = processor.rename_key("md.doc.due_date", "due").unwrap();
    /// let mut chars: Vec<char> = text.chars().collect();
    /// for edit in renamed.edits.iter().rev() {
    ///     chars.splice(edit.start..edit.end, edit.text.chars());
    /// }
    /// assert_eq!(chars.into_iter().collect::<String>(), "Due {{md.doc.due}}.\n\n:: badge[Due] {title=(md.doc['due'])}\n");
    /// assert!(processor.rename_key("md.doc.due_date", "due date").is_err());
    /// ```
    pub fn rename_key(
        &self,
        path: &str,
        key: &str,
    ) -> Result<crate::path_references::KeyRename, crate::path_references::PathError> {
        let path = crate::path_references::MetadataPath::new(path)?;
        crate::path_references::check_key(key)?;
        if self.is_plain_markdown() {
            return Ok(crate::path_references::KeyRename::default());
        }
        let resolved = &self.state.validated.resolved;
        Ok(crate::path_references::rename_references(
            resolved.document(resolved.main),
            &path,
            key,
        ))
    }

    /// The references in the processed document whose label no definition of it defines (`[text][label]`,
    /// `[label][]`, `[label]`), which CommonMark makes text, in written order, each with its label as written and
    /// where the label is: what an editor warns about. In a plain Markdown document too. It parses the text again, so
    /// it costs a parse.
    ///
    /// ```
    /// use compmd::context::RenderContext;
    /// use compmd::{Ambient, ParseOptions, Processor};
    /// use std::rc::Rc;
    ///
    /// let text = "See [the tides][tides] and [the tables][tables].\n\n[tables]: ./tables.md\n";
    /// let context = RenderContext { now: 0, tz: "UTC".into(), time: Rc::new(()), inputs: Default::default() };
    /// let processor = Processor::new("a.comp.md", text)
    ///     .parse(&ParseOptions::default())
    ///     .resolve((), (), &Ambient::default())
    ///     .validate(&[])
    ///     .load(context, (), ())
    ///     .expand(&Default::default());
    /// let [tides] = processor.unresolved_references().try_into().unwrap();
    /// assert_eq!((tides.label.as_str(), tides.position.start.column), ("tides", 17));
    /// ```
    #[must_use]
    pub fn unresolved_references(&self) -> Vec<crate::unresolved_references::UnresolvedReference> {
        let resolved = &self.state.validated.resolved;
        self.state
            .validated
            .edit_context
            .parsed
            .unresolved_references(&resolved.document(resolved.main).text)
    }

    /// The processed document's tree (spec §12.1).
    #[must_use]
    pub fn tree(&self) -> &tree::Root {
        let resolved = &self.state.validated.resolved;
        &resolved.document(resolved.main).root
    }

    /// Whether the document is plain CommonMark and GFM, which CompMD processing never applies to (spec §2): a file
    /// opened as plain Markdown ([`ParseOptions::plain_markdown`](crate::ParseOptions::plain_markdown)) without an
    /// `md` key, or a frontmatter that makes processing not apply (F001, F033).
    ///
    /// ```
    /// use compmd::context::RenderContext;
    /// use compmd::{Ambient, ParseOptions, Processor};
    /// use std::rc::Rc;
    ///
    /// let processed = |text: &str| {
    ///     let context = RenderContext { now: 0, tz: "UTC".into(), time: Rc::new(()), inputs: Default::default() };
    ///     Processor::new("notes.md", text)
    ///         .parse(&ParseOptions { plain_markdown: true, ..ParseOptions::default() })
    ///         .resolve((), (), &Ambient::default())
    ///         .validate(&[])
    ///         .load(context, (), ())
    ///         .expand(&Default::default())
    /// };
    /// let plain = processed("::: box\nText.\n::: /box\n");
    /// assert!(plain.is_plain_markdown(), "a `.md` file without `md`");
    /// assert!(plain.hint_at(5).is_none() && plain.tokens().is_empty(), "`:::` is text");
    /// let compmd = processed("---\nmd:\n  version: 1\n---\n::: box\nText.\n::: /box\n");
    /// assert!(!compmd.is_plain_markdown() && compmd.hint_at(30).is_some());
    /// ```
    #[must_use]
    pub fn is_plain_markdown(&self) -> bool {
        self.state.validated.edit_context.parsed.plain_markdown()
    }

    /// Whether the document has wiki links (spec §2.3): a CompMD document that opts in with `md.syntax`, or a file
    /// opened as plain Markdown whose host turns them on
    /// ([`ParseOptions::wiki_links`](crate::ParseOptions::wiki_links)).
    ///
    /// ```
    /// use compmd::context::RenderContext;
    /// use compmd::{Ambient, ParseOptions, Processor};
    /// use std::rc::Rc;
    ///
    /// let processed = |text: &str, options: &ParseOptions| {
    ///     let context = RenderContext { now: 0, tz: "UTC".into(), time: Rc::new(()), inputs: Default::default() };
    ///     Processor::new("notes.md", text)
    ///         .parse(options)
    ///         .resolve((), (), &Ambient::default())
    ///         .validate(&[])
    ///         .load(context, (), ())
    ///         .expand(&Default::default())
    /// };
    /// let opted = processed("---\nmd:\n  syntax: [wiki-links]\n---\n[[Tides]]\n", &ParseOptions::default());
    /// assert!(opted.has_wiki_links());
    /// let markdown = ParseOptions { plain_markdown: true, wiki_links: true, ..ParseOptions::default() };
    /// assert!(processed("[[Tides]]\n", &markdown).has_wiki_links(), "the host's choice for a `.md` file");
    /// assert!(!processed("[[Tides]]\n", &ParseOptions::default()).has_wiki_links());
    /// ```
    #[must_use]
    pub fn has_wiki_links(&self) -> bool {
        self.state.validated.edit_context.parsed.wiki_links()
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
        let mut pending = Pending {
            packs: self.state.validated.resolved.pending.clone(),
            ..self.state.data.pending.clone()
        };
        self.state.validated.examples.pending(&mut pending);
        pending
    }

    /// The expanded tree (spec §12.3).
    #[must_use]
    pub fn expanded(&self) -> &expanded::Root {
        &self.state.result.root
    }

    /// What this render evaluated of the interpolations and expression params written in the processed document, in
    /// written order ([`crate::evaluations`]): the value each gave first, or none when one of its evaluations failed,
    /// and how many times it was evaluated. One the render didn't reach, as in the branch of an `if` it didn't take,
    /// isn't there.
    ///
    /// ```
    /// use compmd::context::RenderContext;
    /// use compmd::{Ambient, ParseOptions, Processor};
    /// use std::rc::Rc;
    ///
    /// let text = "---\ntitle: Harbour notes\nteam: [Ana, Kenji]\n---\n\n# {{md.doc.title}}\n\n::: each {in=(md.doc.team) as=p}\n{{p}}\n::: /each\n";
    /// let context = RenderContext { now: 0, tz: "UTC".into(), time: Rc::new(()), inputs: Default::default() };
    /// let processor = Processor::new("a.comp.md", text)
    ///     .parse(&ParseOptions::default())
    ///     .resolve((), (), &Ambient::default())
    ///     .validate(&[])
    ///     .load(context, (), ())
    ///     .expand(&Default::default());
    /// let [title, team, person] = processor.evaluations() else { panic!("three") };
    /// assert_eq!((title.at.line, title.at.column), (6, 5));
    /// assert_eq!(title.value.as_ref().unwrap().to_json(), "Harbour notes");
    /// assert_eq!((team.at.column, team.count), (14, 1));
    /// assert_eq!((person.value.as_ref().unwrap().to_json(), person.count), ("Ana".into(), 2));
    /// ```
    #[must_use]
    pub fn evaluations(&self) -> &[crate::evaluations::Evaluation] {
        &self.state.result.evaluations
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

    /// Loads and expands again with `inputs` as the values of example `example`'s inputs (spec §7.8), its number as its
    /// expanded root gives it: what a host does when the reader changes one of an example's inputs. The document's own
    /// inputs, and the other examples', keep their values. Values for an example the document doesn't have are kept,
    /// and used by none.
    ///
    /// ```
    /// use compmd::context::RenderContext;
    /// use compmd::{Ambient, ParseOptions, Processor};
    /// use std::rc::Rc;
    ///
    /// let text = "```compmd {render=output}\n---\nmd:\n  inputs:\n    live: boolean = false\n---\nLive: {{md.input.live}}.\n```\n";
    /// let context = RenderContext { now: 0, tz: "UTC".into(), time: Rc::new(()), inputs: Default::default() };
    /// let processor = Processor::new("a.comp.md", text)
    ///     .parse(&ParseOptions::default())
    ///     .resolve((), (), &Ambient::default())
    ///     .validate(&[])
    ///     .load(context, (), ())
    ///     .expand(&Default::default());
    /// let live = |processor: &Processor<_>| serde_json::to_value(processor.expanded()).unwrap()["children"][0]["document"]["children"][0]["children"][0]["value"].clone();
    /// assert_eq!(live(&processor), "Live: false.");
    /// let set = processor.example_inputs(1, serde_json::json!({ "live": true }).as_object().unwrap().clone());
    /// assert_eq!(live(&set), "Live: true.");
    /// ```
    #[must_use]
    pub fn example_inputs(
        self,
        example: usize,
        inputs: serde_json::Map<String, serde_json::Value>,
    ) -> Processor<Expanded> {
        let Processor {
            mut diagnostics,
            state,
        } = self;
        let Expanded {
            validated,
            mut render,
            options,
            result,
            ..
        } = state;
        render.examples.insert(example, inputs);
        let validated = back(validated, &mut diagnostics);
        let mut processor = Processor {
            diagnostics,
            state: validated,
        }
        .load_with(render)
        .expand(&options);
        processor.state.replaced.nodes = 0..result.root.children.len();
        processor
    }

    /// The host-language components the document uses, with their code, by name: what a renderer that runs them
    /// needs, since the tree names a component but doesn't hold its code (spec §7.4, §12.3). Used is invoked, or
    /// drawing a code block, in the body or in a definition it uses, in any branch or template: so the list doesn't
    /// change with the inputs, which may show a branch the last render didn't. A component the documents could name
    /// but the document never uses, such as most of an ambient pack's, isn't listed, so a host loads no code for it.
    /// Those of an example (spec §7.8) come after the document's, each with its example's number. Only those one of
    /// the processor's bindings runs (others render their fallback, W102), and those the host implements (a signature
    /// fence alone in an ambient pack, spec §7.1), with no language or code; a name two documents give different
    /// components is the processed document's, or else the first loaded.
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
    /// assert_eq!((hello.name.as_str(), hello.language), ("hello", Some(Language::Jsx)));
    /// assert_eq!(hello.code, "export default () => <b>Hi</b>;");
    /// ```
    #[must_use]
    pub fn host_components(&self) -> Vec<HostComponent> {
        let validated = &self.state.validated;
        let mut components = validated.resolved.host_components(validated.store.store());
        components.extend(validated.examples.host_components());
        components
    }

    /// The remote and `host:` sources a render needs with the context the processor keeps, and the packs it has no
    /// release of, as [`sources`](Processor#method.sources) gives them before loading. After an edit to the
    /// frontmatter, a host that fetches ahead fetches what it hasn't yet and calls
    /// [`reload`](Processor#method.reload-1); a pack it gets needs the document processed again with it.
    #[must_use]
    pub fn sources(&self) -> Needed {
        let validated = &self.state.validated;
        let mut needed = load::sources(
            &validated.resolved,
            &validated.checks,
            &self.state.render.context,
            &validated.allowed_origins,
        );
        validated
            .examples
            .sources(&self.state.render.context, &mut needed);
        needed
    }

    /// The release each of the document's own pack imports took, with where its `from` is written, in the order the
    /// imports are: what an editor says about an import by the pack it loaded.
    ///
    /// ```
    /// use compmd::context::RenderContext;
    /// use compmd::host::{LoadedRelease, Release};
    /// use compmd::{Ambient, ParseOptions, Processor};
    /// use semver::Version;
    /// use std::collections::HashMap;
    /// use std::rc::Rc;
    ///
    /// let document = "---\nmd:\n  export:\n    names: \"*\"\n---\n\n```compmd component=note\n---\nA note.\n```\n";
    /// let files = HashMap::from([
    ///     ("package.json".to_string(), r#"{"name": "@kit/notes", "version": "1.0.0"}"#.to_string()),
    ///     ("index.comp.md".to_string(), document.to_string()),
    /// ]);
    /// let release = Release { version: Version::new(1, 0, 0), integrity: None, broken: None };
    /// let packs = HashMap::from([("@kit/notes".to_string(), vec![LoadedRelease { release, files }])]);
    /// let text = "---\nmd:\n  import:\n    - from: \"@kit/notes@1\"\n      names: [note]\n---\n";
    /// let context = RenderContext { now: 0, tz: "UTC".into(), time: Rc::new(()), inputs: Default::default() };
    /// let processor = Processor::new("a.comp.md", text)
    ///     .parse(&ParseOptions::default())
    ///     .resolve((), packs, &Ambient::default())
    ///     .validate(&[])
    ///     .load(context, (), ())
    ///     .expand(&Default::default());
    /// let [(at, release)] = processor.imported_packs().try_into().unwrap();
    /// assert_eq!((at.line, release.to_string()), (4, "@kit/notes@1.0.0".to_string()));
    /// ```
    #[must_use]
    pub fn imported_packs(&self) -> Vec<(Location, ReleaseId)> {
        let resolved = &self.state.validated.resolved;
        resolved
            .document(resolved.main)
            .imported
            .iter()
            .filter_map(|(at, origin)| match origin {
                Origin::Pack(place) => Some((*at, place.release.clone())),
                Origin::File(_) | Origin::Unplaced(_) => None,
            })
            .collect()
    }
}
