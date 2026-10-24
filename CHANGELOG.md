# CompMD spec changelog

Changes to the [CompMD specification](compmd-spec-v1.md), newest first. Every change to the spec adds an entry here, naming the sections it touches. Version 1 is a draft until it is published; until then, a change can break documents written against an earlier draft, and its entry says so.

## Version 1 (draft)

### 2026-10-24

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
