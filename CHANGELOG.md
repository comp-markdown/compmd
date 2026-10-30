# CompMD spec changelog

Changes to the [CompMD specification](compmd-spec-v1.md), newest first. Every change to the spec adds an entry here, naming the sections it touches. Version 1 is a draft until it is published; until then, a change can break documents written against an earlier draft, and its entry says so.

## Version 1 (draft)

### 2026-10-30

- **Code renderers (§1.1, §2.3, §3, §4.5, §5.1, §6.3, §7, §7.3, §7.7, §10.2, §11, §12.1, §12.3, §17).** A leaf whose signature has `code: {language, param}` also renders the code blocks of that language, the block's text as that param: ` ```mermaid ` in a document that imports a `mermaid` code renderer is drawn, and stays a code block in plain renderers and elsewhere. Params go on the info string after the language, read only for rendered blocks; `alt` is the text alternative (W101 without it); two renderers of one language at one level are F023; the `code` param on the info string is F017, and a `code` naming a param that isn't a required `string` is F009. The expanded tree's `code` node gains `component` and `alt`, in the tree schema too. `component=` defines a component only in a definition language. Not breaking: a code block renders differently only in a document that imports a code renderer. Conformance cases come with the processor's support.
- **Any pack, in the strict sandbox (§6.4, §10.2, §15, §17).** Which packs a document may import is the processor's configuration: by default any pack in the registry, whose code runs in the strict sandbox, one sandbox per document for packs the configuration doesn't list. A processor may allow only listed packs, deny packs as its own policy (not for security, which the sandbox gives), and cap a pack's size; a pack it doesn't allow is F026. Packs are fetched by the processor, never by the reader's browser. Before, §15 said remote component imports needed an allowed origin, as remote data does. Not breaking: documents are unchanged.
- **Built modules (§6.4).** A pack's component file may be a built ES module with its libraries bundled in, which a processor may load by its place in the pack instead of passing its text, from files checked against the pack's integrity.
- **Trust levels (§1.1, §7.4, §15, §17).** A pack's code runs at one of three levels the processor's configuration sets: a strict sandbox (the default, the sandbox as before), a permissive sandbox (the network to the origins listed for its pack, and `host`), or trusted (the host's own code, built with its application, outside a sandbox, which treats params and slots as untrusted input). Before, all component code ran sandboxed, and "packs the host trusts" meant what is now a permissive pack. Not breaking: documents are unchanged.
- **Components the host implements (§6.3, §7.1).** A signature fence alone in an ambient pack declares a component whose code the processor provides, validated against the signature; elsewhere it stays F002. An ambient pack imported by name is always the processor's, never a registry package of that name. New cases `expand/host-signature`, `validate/host-signature-params-F008-F009` and `resolve/ambient-import-by-name`.
- **Unknown type names are F037 (§7.3, §10.2).** A declaration whose type isn't in the grammar, such as `count: numbr`, in a signature, an input or a data `schema`, is F037 at the type name, with the nearest type name. The declaration is kept and takes any value. Before, such a type was accepted silently and checked nothing. Breaking for documents with a misspelt type, which now have an error.
- **Website.** The spec links to the CompMD website, which has the spec as a page and a playground. The spec itself is unchanged.

### 2026-10-29

- **Links to a passage (§2.3).** A link to a text fragment (`#:~:text=…`) names a passage of the document by its text; a tool that shows the source, such as an editor beside its preview, may select the passage there. No new syntax: they are ordinary links, and the spec says how such tools may read them.

### 2026-10-27

- **Timestamp and duration getters (§8.2).** The profile's timestamp and duration operations include CEL's getters, `getFullYear` to `getMilliseconds`, so a document can show the time of day. A timestamp's getters take a time zone and, unlike CEL's, use `md.tz` without one. Before, the getters weren't named, and a processor could leave them out.
- **Numbers as text (§8.1, §8.2).** Interpolation and `string()` write a number as JavaScript's `String(n)` does: plain from 10⁻⁶ up to 10²¹, with an exponent outside (`1e+21`, `1.5e-7`), and `NaN`, `Infinity` and `-Infinity`. Before, the form of very large and very small numbers wasn't said.
- **Number map keys match by value (§8.2).** As in CEL, `{1: "a"}[1u]` and `{1: "a"}[1.0]` are `"a"`, so `md.groupBy` on a field of whole numbers from data gives `int` keys.
- **Inline math follows Pandoc's rule in full (§2.3).** Outside a `{…}` group, a `$` with white space on its left ends the attempt, and the opening `$` is text, as in Pandoc. Before, a later valid closer could reach back past it, so `It costs $5 and $6. Math $x^2$` made `5 and $6. Math $x^2` math; now only `$x^2$` is. Breaking for documents that relied on the old reading. New case `base-syntax/math-dollar-after-space`, checked against Pandoc 3.12.1.
- **Code spans in fallbacks (§3.1, §3.2 rule 9).** A code span binds tighter than a fallback's brackets, as in CommonMark's link text: `:a[b `c]` d]` has the fallback ``b `c]` d``, as plain renderers show it. Before, the `]` inside the code span ended the fallback. Breaking for such fallbacks. New case `inline/fallback-code-span`.
- **Terms (§1.1).** A glossary of the spec's terms, each with its section. "Container" means only a `:::` component; the box a block's breakpoints follow is its *nearest box*.
- **Breakpoints follow the nearest box (§9.3).** A block's params follow the width of its nearest box, the term §9.2 uses for positions, rather than its parent's: list items, quotes and alerts don't count, so a component in a list item inside a box follows the box. A new informative §9.5 shows which box breakpoints follow in each case. Only the wording changes, since list items and quotes were never boxes.
- **W114 wherever a box's width comes from its content (§9.3, §10.2).** A box whose blocks use breakpoints shrinks to nothing wherever its width would come from its content. W114 was only for a box in a `row`; it now also covers a box in a `stack` that isn't stretched across it, a box in a `grid` with `justify-self`, and a positioned box without both inline insets. A document may now have W114 where it had none. New cases: `expand/column-child-width-W114`, `expand/grid-child-width-W114`, `expand/positioned-width-W114`, and `expand/block-child-width` with none.
- **Slot items and their width (§7.4).** A host-language component that lays its items out by their content gives an item a width when the block in it uses breakpoints.

### 2026-10-24

- **The reference rendering (§13.1).** The informative HTML and CSS rendering of the built-ins is `@compmd/render`'s, in [`render/`](render/README.md#the-reference-rendering-of-the-built-ins). The spec itself is unchanged.
- **Breakpoints follow the parent's width (§9.3, §10.2; W114).** Every param of a block, its layout among them, follows its parent's width; before, a block's own layout followed its own width, which CSS can't express (an element's styles follow an ancestor's width, not its own). The two differ only for a block much narrower than its parent: `::: grid {width=40% md:grid-template-columns="1fr 1fr"}` on a wide page now has two columns. W114 is for a block in a `row` whose *children* use breakpoints, as §9.3 said; a block whose own layout uses breakpoints no longer gets it. Case `expand/row-child-width-W114` is updated.
- **Origins (§12.3, §13.1, §13.2, §17).** A new optional feature, `origins`: every expanded node has `origin`, the range of the processed document that made it, with its `occurrence` among the nodes before it with the same range. A node from a definition's body takes its invocation's origin, so an editor can keep what it rendered for a node through an edit. Keys for matching nodes across edits are the editor's, not the format's. The tree schema has `origin`, and a new case is `expand/origins`.
- **Record keys, an open issue (§17).** `each` iterations and template slot instances are told apart by index, not by record, so state shown for one record moves to another when records are added in front. A record key is to be decided.
- **Author.** The spec names its author.

### 2026-10-23

- **The AI authoring guide.** [`llms.txt`](llms.txt) is what an AI agent reads before writing CompMD. Measured with Claude Sonnet and Haiku on 12 tasks: 12 of 12 valid on the first try for each, against 0 of 12 without it (see the [README](README.md#the-ai-authoring-guide)). The spec itself is unchanged.
- **Tree schema (§12.4).** [`conformance/compmd-tree.schema.json`](conformance/compmd-tree.schema.json) defines the parse tree and the expanded tree; every expected tree in the conformance set fits it.
- **Inline math (§2.3).** The rule is stated in full: a `$` inside an open `{` group doesn't close math, as in Pandoc, and the differences from GitHub are listed. Breaking: `{{$doc.title}} by {{$doc.author}}` is no longer math. New case `base-syntax/math-dollar-boundaries`.
- **Renamed to CompMD (all sections).** Breaking for every document.
  - The format is CompMD (Component Markdown), and files are `.comp.md`.
  - The term is "component" throughout, and node types are `componentContainer`, `componentLeaf`, `componentInline`, `componentSlot`, `componentDefinition`, `componentSignature` and `interpolation`.
  - Definition fences are ` ```compmd component=<name> `.
  - The frontmatter key is `md:`, and the expression namespace is `md.` (`md.doc`, `md.data`, `md.input`, `md.slot`, `md.currency(…)`, …).
  - The built-in pack is `@compmd/core`, with `@compmd/*` reserved.
  - Other renames: the `compmd` field in `package.json`, the `compmd` prop of the React and Svelte bindings, the `--compmd-*` CSS custom properties, "component line" and "CompMD processor".
  - Diagnostic codes are unchanged.

### 2026-10-22

- **A component is one box (§7, §7.2, §9, §9.1, §9.2, §12.1, §12.3; F036, W120).** Breaking: definitions that wrapped their body in `::: box {…}` now get two boxes.
  - Every block component is one box, styled by a param list on its definition fence and overridden by the caller's params as a later CSS cascade layer.
  - `display=contents` gives a component no box.
  - `stack`, `row` and `grid` set their layout on their fences.
  - This replaces the wrapper box made only when a component was given common params.
- **Inputs in the expanded tree (§12.3).** An input-bound param carries the input's value and name, and the root lists the document's inputs, so the expanded tree is a renderer's whole input.
- **Layout is `@compmd/core` (§9).** `box`, `stack`, `row` and `grid` with params named after CSS properties, common params, positioned layout and visual tokens. Component files in plain JavaScript and TypeScript too (§6, §7.1). Documents a processor shows as one application MAY share a sandbox, and trusted components act within their configuration in its own documents (§15).
- **Findings from the parser, the processor and the examples.** `input<enum>`, missing imports and bad breakpoints; expansion of definitions, control flow and data; list functions, reserved names and the changelog example.
- **Open issues settled:** base syntax, raw HTML and extensions; host-language components; template slots; the CEL profile; data imports and host data; layout and visual styling; security and packages; accessibility and text direction; smaller points.

### 2026-10-21

- **Open issues settled:** GitHub READMEs and live documents; conformance, the document tree and the test format; grammar precision; frontmatter and versions; names; diagnostics, recovery and fallbacks.
- **The conformance set**, pass 1 (stage 1).
- **§16** is a short release-notes example, and data `from` values can interpolate inputs.
- **The first draft.**
