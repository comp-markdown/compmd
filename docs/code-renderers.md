# Code renderers

**Status: accepted (2026-10-30).** Specified in spec §7.7, with one change from this design: the signature says `renders: mermaid` (or a list of names of one language), and the block's text is always the `code` param, which needn't be declared, instead of `code: {language, param}`. Part of [components from apps and packs](components-from-apps-and-packs.md), where it is option W2 for Mermaid.

A **code renderer** is a component that also renders the code blocks of one language: a pack's `mermaid` draws every ` ```mermaid ` block of a document that imports it. The document writes the block as it would anywhere:

````md
```mermaid
graph LR; Draft --> Review --> Published
```
````

A plain renderer shows the code, and GitHub draws the diagram itself.

## Fences and components, today

The spec uses fences for two things, and neither invokes a component:

- **A code block** (` ```mermaid `, ` ```sh `) is content. A processor MAY render it by its language, as GitHub renders Mermaid (§2.3), and `@compmd/render` takes a function per language for it (`code: { mermaid: … }`).
- **A definition fence** (` ```jsx component=chart `, ` ```compmd component=card signature `) defines a component or its signature (§7). Processors remove it from the output.

Components are invoked only with `:::`, `::` and `:` (§3): every component, in every document.

**What this adds** is a third case of the first: a pack can supply the renderer of a code block's language, as a host already can. The block stays a code block in the document and in plain renderers; it is never a way to invoke other components, and only a language a pack names renders this way.

## A leaf, a container or a code block

A component author already chooses between a container (it takes Markdown) and a leaf (it takes params and nothing else). Code renderers add a third form: a code block, which takes source text in a language of its own.

| The content is | The form | Examples |
|---|---|---|
| Params only, on one line | a leaf | `:: stars[4 of 5] {value=4}`, `:: chart[Sales] {rows=(md.data.sales)}` |
| Markdown, which plain renderers should show as Markdown | a container | `::: card`, `::: tabs`, `::: callout` |
| Source in a language that exists outside CompMD, which plain renderers should show as source | a code block | ` ```mermaid `, ` ```dot ` (Graphviz), ` ```abc ` (music), ` ```vega-lite ` (a chart as JSON), ` ```geojson ` (a map; GitHub draws it) |
| Source computed from data or inputs | the same component as a leaf, its source as a param | `:: mermaid[Our process] {source=(md.data.steps.map(s, s.name).join(" --> "))}`, since a code block is never interpolated (§8.1) |

The test for a code block: would a reader of the plain Markdown want to see that source, and does a tool outside CompMD already know the language? If not, it's a leaf or a container.

**The closest thing in the spec today is math.** CommonMark has no math, and neither does the GFM spec; GitHub renders ` ```math ` blocks and `$…$` as a feature of its own. CompMD made GitHub's math part of its base syntax (§2.3), so every CompMD parser reads a ` ```math ` block as a `math` node, not a code block. That is a language name giving a code block a meaning, as a code renderer does, with two differences: math is fixed by the spec for every processor, and it changes the tree at parsing; a code renderer comes from a pack the document imports, and the block stays a code block until it is rendered.

## One component, both forms

A code renderer is an ordinary leaf whose signature also names a language and the param its code fills:

````md
<!-- @kit/mermaid's index.comp.md -->
---
md:
  import:
    - from: ./dist/mermaid.js
      names: [mermaid]
  export:
    pack: "@kit/mermaid"
    version: "1.0.0"
    names: [mermaid]
---

```compmd component=mermaid signature
code: {language: mermaid, param: source}
params:
  source: string
  theme: default | dark | neutral = default
```
````

```ts
// src/mermaid.ts: one mount function for both forms
export default function mount(target, { params, fallback }) {
  draw(target, params.source, params.theme, fallback.text);
  return { update: (next) => draw(target, next.source, next.theme, fallback.text) };
}
```

A document uses either form, with one import:

````md
---
md:
  import:
    - from: "@kit/mermaid@1"
      names: [mermaid]
  data:
    - from: ./steps.yaml
      as: steps
---

```mermaid {alt="Draft, then review, then published"}
graph LR; Draft --> Review --> Published
```

:: mermaid[Our process, from steps.yaml] {source=("graph LR; " + md.data.steps.map(s, s.name).join(" --> "))}
````

In the code block, the block's text is the `source` param, and `alt` is the fallback text; in the leaf, both are written as for any leaf. The component can't tell the two apart, and needs no new part of the host contract.

**Challenged: one name for both forms, against a code renderer and a leaf apart** (the first draft of this document: `mermaid` with `code: mermaid` and a new `code` in the contract, and a `diagram` leaf beside it).

| | One component, both forms | A renderer and a leaf apart |
|---|---|---|
| Signatures | one | two, with the same params repeated |
| What a document imports | `mermaid` | `mermaid` and `diagram` |
| The host contract | unchanged: the code is a param | a new `code` field, in `mount` and `update` |
| A component that is only a code renderer | still a leaf too, with its source as a param: harmless, and useful for computed source | possible |
| The source param | must be a `string` (F009 on the signature otherwise), and is required | none |
| A param a block can't give | the source param on the info string is F017, given twice | none |
| Fallback text | the leaf's `[…]`, or the block's `alt` | each its own |
| Renaming on import | `{name: mermaid, as: diagram}` renames the leaf; ` ```mermaid ` blocks still render, since the language is the signature's | each its own |

One component is simpler everywhere except one case: a code renderer that should never be a leaf. No such case has come up, and a leaf with a source param does no harm, so the design takes one component.

## The questions, with proposed answers

1. **Is it a component?** Yes: a leaf whose signature has `code: {language, param}`. The language chooses it for code blocks; its name is what a document imports and what diagnostics name.
2. **How a document gets it.** By importing the component, as any other: `names: [mermaid]`, or `"*"`. A ` ```mermaid ` block in a document that imports none stays a code block, with no diagnostic, since most code blocks are just code. An ambient pack may provide one, at the lowest priority (§6.3).
3. **Two renderers for one language.** Two imports whose components both render `mermaid` are F023 when a ` ```mermaid ` block is used, as two wildcard imports of one name are; listing one explicitly resolves it.
4. **Params on the info string.** CommonMark leaves everything after the language to the writer, and tools use it in different ways (Pandoc's `{.class}`, a `title="…"`). Proposed: a param list in braces after the language, with the param grammar of §3.1: ```` ```mermaid {theme=dark width=50%} ````. Without a code renderer, the info string's rest is kept, as now. Only definition languages (§7) take `component=`; ```` ```mermaid component=x ```` is an ordinary code block whose info string has `component=x` in it.
5. **The code's text.** The param `code.param` names, with the block's text as its value. No change to the host contract.
6. **The box.** A rendered code block has an outer box, as the leaf has, so the common params apply (`width`, `padding`, `border`, breakpoints).
7. **Fallback and accessible text.** The fallback is the code block itself (§4.5): what a plain renderer shows. The text alternative a leaf writes in `[…]` is written `alt` on the block, a param every rendered code block takes, as every component takes `class` and `id`; it becomes `fallback.text`. W101 when it's missing, as for a leaf without fallback text.
8. **No interpolation.** `{{md.doc.x}}` in a ` ```mermaid ` block stays as written (§8.1); computed source is the leaf's job.
9. **The host's own renderers.** `@compmd/render`'s `code` option stays: a host that renders `mermaid` itself is an ambient code renderer, and a document's import wins over it (§6.3).
10. **No inline form.** CommonMark has no language for a code span, so code renderers render blocks only.

## What it means for the spec

| Section | Change | Conflict with today's text |
|---|---|---|
| §2.3 | code blocks of a language rendered by an imported code renderer, next to "a processor MAY render a code block by its language" | none: it extends it |
| §3 | the param list on a code block's info string | §3 lists every CompMD form; this is a sixth, inside a fence's info string, which parsing reads only for code renderers |
| §4.5 | a rendered code block's fallback is the code block | none |
| §5.1 | `alt` on a rendered code block, as `class` and `id` on any component | none |
| §6.3 | resolution by language; F023 for two renderers of one language | none |
| §7, §7.3 | `code: {language, param}` in a signature, its param a required `string`; `component=` only on definition languages | §7 says a fence whose info string has `component=` defines a component; it needs "in a definition language" |
| §7.4 | none: the code is a param, `alt` the fallback text | none |
| §8.1 | no interpolation in code blocks, as now | none |
| §12.3 | a code block in the expanded tree carries its component and params, as a leaf does | the schema's `code` node gains fields |
| §10.2 | W101 for a rendered code block without `alt`; F017 for the source param on the info string | W101 names leaves only |

## Content in another language (an idea, for later)

A code renderer's signature says its block's content is Mermaid. The same declaration can tell an editor how to treat it: highlighting, completion, diagnostics. A signature could name the language of a code block it renders (`code: {language: mermaid}`) or of a slot's content, and a pack could ship what an editor needs for it: a TextMate or tree-sitter grammar for highlighting, and optionally a language server for completion. An editor that knows CompMD would then highlight a Mermaid block in any document that imports the pack, without knowing Mermaid itself. This is its own design, after code renderers.
