# CompMD compared with existing specs

How the CompMD spec (version 1 draft) compares with seven Markdown-based formats that add components, expressions or interactivity:
- MDX
- Markdoc
- Nuxt MDC and Comark
- MyST
- Quarto
- Livebook's Live Markdown
- Active Markdown

Researched 2026-10-23 from each project's docs, specs, repositories and package registries. It compares the spec as it now stands.

**How it was checked (2026-10-23):**
- Versions, dates, stars and download counts come from the npm, PyPI and GitHub APIs.
- Quotes and behaviour come from each project's own docs, specs and source.
- The plain-renderer column comes from rendering samples through GitHub's Markdown API (`gfm` mode), shown under [Plain-renderer samples](#plain-renderer-samples).

What no source settles is marked *not documented*.

## Summary

**What only CompMD has.** No other format combines:
- component definitions written in the document, in a template language with typed signatures (§7.2, §7.3);
- a safe, typed expression language (the CEL profile, §8.2);
- `if` and `each` as document constructs;
- declared data imports and typed inputs (§8.6, §8.7);
- a layout vocabulary with CSS's meaning (§9);
- components from React, Svelte or plain JavaScript through one host contract (§7.4);
- a written degradation contract for plain renderers (§14).

**Closest relatives:**
- **Markdoc** shares the philosophy: declarative documents, no code in content, schema-typed attributes and coded diagnostics. But it has no in-document definitions, no operators in expressions, no loops and no data.
- **Nuxt MDC and Comark** share the syntax family (`::name{…}`, `#slot`) and the idea of host components from several frameworks. But they have no types, no spec and no degradation contract.
- **MyST** is the only other format with a written spec, a JSON schema and a test set. Its definitions live in plugins, and it has no control flow.
- **Quarto** has the richest layout and real reactive inputs (Observable). Its components are untyped Lua extensions, and rendering runs the author's code.

**Where others are ahead:**
- Ecosystem size: MDX (about 7.3M npm downloads a week), Quarto and MyST.
- Output range: Quarto, through Pandoc.
- Notebook-style computation: Livebook, Quarto and MyST.
- Streaming: Comark's auto-close for partial input.
- Process: MyST's enhancement proposals, and Quarto 2's formal grammar with source maps.

[What the spec could take from them](#what-the-spec-could-take-from-them) lists these, without deciding anything.

## The formats

| Format | Owner | Version (2026-10) | License | Adoption |
|---|---|---|---|---|
| **CompMD** | This repository | v1 draft | Text CC BY 4.0, tests MIT | Not yet published |
| **MDX** | mdx-js (main maintainer Titus Wormer; © Compositor and Vercel) | 3.1.1 (major 3 since 2023-10) | MIT | About 19.8k stars, about 7.3M npm downloads a week; Next.js, Astro, Docusaurus and others |
| **Markdoc** | Stripe | 0.5.10 (2026-09) | MIT | About 8.5k stars, about 427k downloads a week; Stripe's docs |
| **Nuxt MDC / Comark** | NuxtLabs; Comark maintained by Vercel (per comark.dev) | `@nuxtjs/mdc` 0.23.2 (being deprecated for Comark); `comark` 0.8.1 | MIT | MDC about 150k downloads a week (Nuxt Content); Comark about 1.1k stars, about 64k a week |
| **MyST** | Jupyter Book / Executable Books | `mystmd` 1.11.0; `myst-parser` 5.1.0 | MIT | myst-parser about 1.35M PyPI downloads a week; Jupyter Book 2 is built on it |
| **Quarto** | Posit | 1.10.19 (2026-10-06); Quarto 2 in development (`quarto-dev/q2`, "the experimental implementation of Quarto 2 in Rust") | MIT (Pandoc GPL-2.0) | About 6.1k stars; R and Python data science |
| **Live Markdown** (`.livemd`) | Livebook (Dashbit) | Livebook 0.19.10 | Apache-2.0 | About 5.9k stars; GitHub renders `.livemd` |
| **Active Markdown** | Alec Perkins | 0.4.2 (2024 rewrite) | MIT | 62 stars, about 34 downloads a month; dormant |

## Specification and conformance

| Format | Written spec | Conformance tests | Implementations |
|---|---|---|---|
| **CompMD** | Yes: a normative draft with RFC 2119 wording, grammar, a diagnostic catalog and a processing model | Yes: 166 cases (input, expected parse or expanded tree, diagnostics) by area | One prototype (TypeScript) runs the full set |
| **MDX** | No. The spec repo was archived in 2024; the micromark extensions are the reference | No shared suite; per-implementation tests | JavaScript (`@mdx-js/mdx`), Rust (`mdxjs-rs`) |
| **Markdoc** | Draft "0.1.0", "largely a work in progress"; covers tag syntax only, not evaluation | No; internal YAML-driven tests | JavaScript (official), Rust port (`accent_proust`, with a list of divergences) |
| **MDC / Comark** | No | No; Vitest suites | JavaScript/TypeScript only |
| **MyST** | Yes: markup, an mdast-based AST with a JSON Schema, and test cases including all CommonMark examples. Explicitly unstable. | Test cases, but no formal conformance process | TypeScript (`mystmd`), Python (`myst-parser`) |
| **Quarto** | Quarto 1: no (Pandoc Markdown plus a pre-parser). Quarto 2: "a formal grammar" in Tree-sitter, per the team's 2026-05 post; no standalone spec published. | No | TypeScript/Deno plus Pandoc and Lua; Quarto 2 in Rust (experimental) |
| **Live Markdown** | No: a module doc and the import/export code | Livebook's own tests | Livebook only |
| **Active Markdown** | A short prose reference with MUST/MAY | Vitest suites | One CLI |

## Syntax

| Format | Base | Block component | Inline component | Named slots | Params |
|---|---|---|---|---|---|
| **CompMD** | CommonMark + GFM, plus alerts and math; nothing removed. Files `.comp.md`. | `::: card {title="Setup"}` … `::: /card`, and leaves `:: chart[fallback] {…}` | `:badge[beta]{tone=info}` | `:: #footer` lines | `key=value`, quoted strings, JSON, `(CEL)`, flags, `.class`, `#id`, breakpoint prefixes; typed and validated against signatures (F008–F010) |
| **MDX** | CommonMark with removals: no indented code, autolinks or HTML (JSX instead); `<` and `{` must be escaped. GFM opt-in. `.mdx`. | `<Callout type="warning">` … `</Callout>` | `<Badge count={3} />` | No; JSX passed through a prop | JSX attributes: strings or `{JS}`; untyped at the MDX level |
| **Markdoc** | CommonMark via markdown-it, with indented code, setext headings and raw HTML disabled | `{% callout type="warning" %}` … `{% /callout %}` | A tag on one line | Opt-in `{% slot "heading" %}` (0.5.10) | `key=value` with strings, numbers, booleans, null, JSON, `$var`, function calls; `#id`, `.class`; typed and validated against the tag schema (`type`, `required`, `matches`, `errorLevel`) |
| **MDC / Comark** | CommonMark + GFM with documented deviations (e.g. no tight/loose lists, no shortcut reference links) | `::alert{type="info"}` … `::` | `:badge[New]{color="blue"}` | `#name` lines | `{key="v" bool .c #id}`, `:key='json'`, YAML props in the body; untyped |
| **MyST** | CommonMark superset; GFM features opt-in | `:::{note} arg` with `:key: value` options and a body | `` {abbr}`text` `` roles | No; child directives | Options typed per directive (string, number, boolean, parsed; `required`) |
| **Quarto** | Pandoc Markdown (not CommonMark). `.qmd` | `::: {.callout-note}` divs; `{{< shortcode >}}` | `[text]{.class}` spans; shortcodes | No | Div attributes and shortcode arguments as Pandoc inlines; untyped |
| **Live Markdown** | A Markdown subset; `#` is the title, `##` sections; metadata in `<!-- livebook:{…} -->` comments | None; code cells are Elixir fences | None | No | Smart-cell state as base64 JSON in a comment |
| **Active Markdown** | GFM via `marked`, plus a regex pre-pass | None | `[5 cookies]{n: 1..10 by 0.5}` (fixed set: string, range, switch, chart) | No | A fixed mini-grammar per control |

## Defining components and packaging

| Format | Where components are defined | Packaging |
|---|---|---|
| **CompMD** | **In the document:** ` ```compmd component=card ` fences in CompMD's own template language, with a typed signature (params, enums, `input<…>`, slots, template slots). Also React, Svelte or plain JavaScript in host-language fences or component files, through the host contract (§7.4, §7.6). | Imports from files and versioned packs (`@scope/pack`, npm packages with `integrity` hashes, §6); ambient packs a processor provides |
| **MDX** | In the document as JS (`export const X = …`), imported via ESM, or passed in by the host | npm |
| **Markdoc** | Host JS config only: tag schemas plus React components; partials for reused content | npm |
| **MDC / Comark** | Host framework components (Vue, React, Svelte, Angular), mapped by name or a lazy manifest | npm |
| **MyST** | JS plugins, executable plugins (any language, AST over stdin/stdout) or Sphinx Python extensions | Plugins listed in `myst.yml` (`project.plugins`); no versioning documented |
| **Quarto** | Lua shortcodes and filters | `quarto add owner/repo` into `_extensions/`; `quarto-required` version |
| **Live Markdown** | Elixir (Kino: `Kino.JS`, `Kino.JS.Live`, `Kino.SmartCell`) with raw JS UIs | Hex packages via `Mix.install` |
| **Active Markdown** | Not possible: four built-in controls | None |

## Expressions, control flow and data

| Format | Expressions | Control flow | Data |
|---|---|---|---|
| **CompMD** | A typed **CEL profile**: not Turing-complete, no I/O. Function list fixed by the spec (formatting, dates, lists, plurals). In `(…)` param values and `{{path}}` interpolation. | `if` (with `#else`) and `each` (join modes, `#empty`, `#before`) | Declared in frontmatter: files (CSV, JSON, YAML) with schemas, HTTPS URLs with caching and fallbacks, `host:` sources; typed records (F026, F027) |
| **MDX** | Full JavaScript in `{…}` | JS expressions (`&&`, `.map`) | ESM imports, host props, top-level `await` |
| **Markdoc** | `$var` paths and function calls; **no operators**; built-ins `equals`, `and`, `or`, `not`, `default` | `{% if %}` / `{% else %}`; **no loops** | Only host-supplied variables |
| **MDC / Comark** | MDC `{{ $doc.title \|\| 'x' }}`; Comark: dot paths with a literal `\|\|` fallback (opt-in plugin) | Comark `::if{:value=… gte=18}` with `eq`, `neq`, `gt`, `gte`, `lt`, `lte` and an `#else` slot; no loops documented | Host `data` prop only |
| **MyST** | myst-parser: Jinja2 substitutions; mystmd: kernel-evaluated `` {eval}`expr` `` | None (an open issue) | `{include}`, `{embed}`, notebook outputs; code cells |
| **Quarto** | `{{< meta >}}`, `{{< var >}}`, `{{< env >}}` lookups; computation in code cells or OJS | Conditional content by output format or boolean metadata; no loops | Code cells, OJS `FileAttachment`, `_variables.yml` |
| **Live Markdown** | Elixir in code cells; prose can't interpolate | In code only | Elixir code, smart cells (SQL), file entries |
| **Active Markdown** | JavaScript in indented code blocks, run with `Function()` | JS inside code blocks only | Inline ` ```csv=name ` and ` ```json=name ` fences |

## Interactivity and layout

| Format | Inputs and state | Layout |
|---|---|---|
| **CompMD** | **Typed inputs declared in frontmatter** (enum, date, number, string, boolean), input components bound to them, and `input<…>` params through which a component changes an input; the processor re-renders (§8.5, §8.7) | **A CSS-like vocabulary:** `box`, `stack`, `row`, `grid` (`min` cards or exact areas); common params for size, padding, borders, positions (`inset-*`), `scroll`, `fill`, `sticky`; breakpoints `sm`/`md`/`lg`; each param means its CSS property (§9) |
| **MDX** | The components' own state (React and so on) | None; components and CSS |
| **Markdoc** | None; React components | None |
| **MDC / Comark** | None; host components. Comark streams partial input (AI chat) | None |
| **MyST** | Jupyter execution, `{anywidget}` (experimental) | `{grid}`, `{card}`, `{tab-set}`, `{dropdown}`, margins |
| **Quarto** | OJS (`viewof x = Inputs.range(…)`, a reactive runtime in the browser), Shiny (a server), Jupyter widgets | The richest: `layout-ncol`, `layout="[[1,1],[1]]"`, article columns, panels, dashboards with rows, columns and cards |
| **Live Markdown** | Kino inputs (text, number, select, date, file and others), forms, dependency-tracked re-evaluation, multi-user | `Kino.Layout.grid` and `tabs`, from code |
| **Active Markdown** | Sliders, switches, HTML inputs, editable code and data; every change re-runs all code | None |

## Plain renderers and security

| Format | In a plain renderer (GitHub) | Code execution and trust |
|---|---|---|
| **CompMD** | **A written contract (§14):** fallbacks for leaves and inline components, containers' content in order with one literal component line around it, definitions as code blocks; the prototype checks every conformance case against a CommonMark and a GFM renderer | **No document runs code.** CEL is safe. Host-language components MUST run sandboxed: only the host contract, no network, no session; what a host gives trusted components (`host`) comes from the processor's configuration, never a document (§15) |
| **MDX** | No contract. GitHub doesn't render `.mdx` files as Markdown (github/markup renders `.md`, `.markdown`, `.mdown`, `.mkdn`; Linguist highlights `.mdx` as source). MDX in a `.md` file: `import`/`export` lines become a paragraph, `{year * 2}` shows literally, and JSX tags are dropped as raw HTML, `<Badge count={3} />` with them | Runs arbitrary JS: "MDX is a programming language. If you trust your authors, that's fine." |
| **Markdoc** | No contract; `{% %}` tags and annotations show literally (`# Title {% #overview %}`), and prose inside a tag renders | No code in documents; only host-registered tags and functions run |
| **MDC / Comark** | No contract; `::alert{type="info"}`, `::`, `#footer` and `:badge[New]{…}` show literally, and the body renders | No code in documents; Comark's security plugin strips `on*` attributes and blocks `javascript:` URLs |
| **MyST** | Guidance only: colon fences have "better fallback" (the fence line shows literally and the body renders); backtick directives show as a code block; roles show as `{abbr}` plus code | Builds run kernels, plugins and Jinja; anywidget runs arbitrary JS |
| **Quarto** | No contract; `::: {.callout-note}` and `{{< meta version >}}` show literally, `{python}` cells show as code without output; the answer is rendering to GFM | Runs code cells, Lua and extensions: "trust the author" |
| **Live Markdown** | Implicit: comments are hidden, cells show as code, saved outputs look like code, inputs vanish | Opening runs arbitrary Elixir; stamps signed with a machine key gate secrets and files |
| **Active Markdown** | `[7 wives]{wives: 1..10}` shows literally; the bracket text doubles as the value | Runs JS through `Function()`; raw HTML and scripts pass through |

## Output and diagnostics

| Format | Tree and output | Diagnostics |
|---|---|---|
| **CompMD** | **mdast-based parse tree and expanded tree** (§12), serialized as JSON; a processor renders any output (HTML, PDF, terminal) by implementing `box` | **A coded catalog**: F001–F036 errors and W101–W120 warnings with exact positions (line and column in code points), related positions, a machine format (§10.5), lenient recovery rules (§10.3) and stages (§11) |
| **MDX** | mdast → hast → JS module (`MDXContent`); React, Preact, Vue, Solid and others | Throws on a syntax error, "with information about where it happened, what occurred, and what was expected" (recovery not documented); component errors at runtime |
| **Markdoc** | Its own AST → a serializable render tree → HTML or React | Coded `ValidateError`s (`missing-closing`, `attribute-type-invalid` and others) with levels and lines; an unclosed tag still yields a tree that `validate` reports on |
| **MDC / Comark** | MDC: mdast/hast; Comark: compact tuples; Vue, React, Svelte, Angular, HTML, ANSI, Markdown | Line numbers only; no error API; auto-closes unterminated syntax |
| **MyST** | MyST AST (extends mdast), JSON Schema; HTML, PDF (LaTeX, Typst), docx, JATS | Typed warnings (`myst.directive_unknown`), line numbers; never fails |
| **Quarto** | Pandoc AST; HTML, PDF, Word, slides, sites, dashboards | Quarto 1: YAML schema errors only, Markdown silent. Quarto 2: coded errors (`Q-2-5`) with source maps; fails rather than recovers |
| **Live Markdown** | Rendered by Livebook; apps; export to an Elixir script | Import warnings; malformed metadata JSON raises |
| **Active Markdown** | One self-contained HTML file | Console only |

## Plain-renderer samples

Each sample rendered with GitHub's Markdown API (`gfm` mode), summarized:

| Format | Sample | On GitHub |
|---|---|---|
| MDX | `import {Chart} from './chart.js'`, `<Callout type="warning">…</Callout>`, `<Badge count={3} />`, `{year * 2}` | The import is a paragraph; the `Callout` tags are dropped and the body shows as raw text (`Some **markdown** inside.`, unrendered); `<Badge … />` disappears; `{year * 2}` shows literally |
| Markdoc | `# Title {% #overview %}`, `{% callout %}…{% /callout %}`, `{% $user.name %}` | Tags show literally; the body's Markdown renders |
| MDC / Comark | `::alert{type="info"}`, `#footer`, `::`, `:badge[New]{color="blue"}`, `{{ $doc.title }}` | All markers literal; the body renders |
| MyST | `:::{note}` with `:class:`, ```` ```{figure} ````, `` {abbr}`…` `` | The colon fence and option lines literal, body renders; the figure is a code block; the role is `{abbr}` plus inline code |
| Quarto | `::: {.callout-note}`, `{{< meta version >}}`, `[text]{.highlight}`, ```` ```{python} ```` | Markers literal; the cell is a code block with no output |
| Active Markdown | `[3 cookies]{cookies: 0..100}` | Literal |

The spec's own degradation contract (§14) is checked the same way by its prototype, for every conformance case, with a CommonMark and a GFM renderer.

## What the spec could take from them

Candidates for the spec's review, not decisions:

- **A JSON Schema for the trees.** MyST publishes one with its AST, and this spec promises one (§12.4) that doesn't exist yet.
- **Diagnostic levels.** Markdoc lets a schema set an attribute's `errorLevel`. This spec's codes are fixed as errors or warnings; that is simpler and probably right, but worth stating as a choice.
- **An enhancement-proposal process** after version 1, as MyST has (MEPs).
- **A formal grammar.** Quarto 2 moved to one (Tree-sitter) for stable parsing and source maps. This spec's grammar is ABNF-style prose; a machine-readable grammar would help editors.
- **Streaming input.** Comark auto-closes unterminated syntax so that partial input (an AI's response) renders while it arrives. This spec's recovery rules (§10.3) already auto-close containers; whether they suit streaming is untested.
- **Output to plain Markdown.** Quarto renders to GFM as its degradation answer. A processor rendering expanded trees back to plain Markdown would complement §14.
- **Metadata in comments.** Livebook keeps structured metadata in `<!-- … -->` comments that plain renderers hide. The spec deliberately keeps everything visible (§14); worth noting as the alternative it rejected.
- **The fallback as the value.** Active Markdown's `[7 wives]{…}` shows a meaningful default wherever it isn't rendered, much as this spec's leaf fallbacks do.

## Sources

**MDX**
- [mdxjs.com: What is MDX](https://mdxjs.com/docs/what-is-mdx/)
- [Getting started (security)](https://mdxjs.com/docs/getting-started/)
- [Using MDX](https://mdxjs.com/docs/using-mdx/)
- [`@mdx-js/mdx`](https://mdxjs.com/packages/mdx/)
- [MDX 3](https://mdxjs.com/blog/v3/)
- [archived spec](https://github.com/mdx-js/specification)
- [mdxjs-rs](https://github.com/wooorm/mdxjs-rs)
- [repository](https://github.com/mdx-js/mdx)

**Markdoc**
- [overview](https://markdoc.dev/docs/overview)
- [syntax](https://markdoc.dev/docs/syntax)
- [spec draft](https://markdoc.dev/spec)
- [tags](https://markdoc.dev/docs/tags)
- [attributes](https://markdoc.dev/docs/attributes)
- [validation](https://markdoc.dev/docs/validation)
- [variables](https://markdoc.dev/docs/variables)
- [functions](https://markdoc.dev/docs/functions)
- [partials](https://markdoc.dev/docs/partials)
- [FAQ](https://markdoc.dev/docs/faq)
- [slots discussion](https://github.com/markdoc/markdoc/discussions/342)
- [accent_proust](https://docs.rs/accent-proust)

**MDC and Comark**
- [nuxt-content/mdc](https://github.com/nuxt-content/mdc)
- [Nuxt Content Markdown](https://content.nuxt.com/docs/files/markdown)
- [comarkdown/comark](https://github.com/comarkdown/comark)
- [components](https://comark.dev/raw/syntax/components.md)
- [Markdown differences](https://comark.dev/raw/syntax/markdown.md)
- [binding](https://comark.dev/raw/plugins/built-in/binding.md)
- [security](https://comark.dev/raw/plugins/built-in/security.md)
- [document model](https://comark.dev/raw/getting-started/document-model.md)
- [migration from MDC](https://comark.dev/raw/kb/migration-from-mdc.md)

**MyST**
- [spec](https://mystmd.org/spec)
- [syntax overview](https://mystmd.org/guide/syntax-overview)
- [JavaScript plugins](https://mystmd.org/guide/javascript-plugins)
- [executable plugins](https://mystmd.org/guide/executable-plugins)
- [widgets](https://mystmd.org/guide/widgets)
- [cards, tabs, dropdowns](https://mystmd.org/guide/dropdowns-cards-and-tabs)
- [exports](https://mystmd.org/guide/documents-exports)
- [myst-parser optional syntax](https://myst-parser.readthedocs.io/en/latest/syntax/optional.html)
- [conditional content issue](https://github.com/executablebooks/mystmd/issues/1280)

**Quarto**
- [Markdown basics](https://quarto.org/docs/authoring/markdown-basics.html)
- [shortcodes](https://quarto.org/docs/extensions/shortcodes.html)
- [extensions](https://quarto.org/docs/extensions/)
- [variables](https://quarto.org/docs/authoring/variables.html)
- [conditional content](https://quarto.org/docs/authoring/conditional.html)
- [OJS](https://quarto.org/docs/interactive/ojs/)
- Layout:
  - [figures](https://quarto.org/docs/authoring/figures.html)
  - [article layout](https://quarto.org/docs/authoring/article-layout.html)
  - [dashboards](https://quarto.org/docs/dashboards/layout.html)
- [Quarto 2 parsing](https://opensource.posit.co/blog/2026-05-07_quarto-2-parsing/)

**Live Markdown**
- [Livebook](https://github.com/livebook-dev/livebook)
- [`Livebook.LiveMarkdown`](https://github.com/livebook-dev/livebook/blob/main/lib/livebook/live_markdown.ex)
- [stamping](https://livebook.hexdocs.pm/stamping.html)
- Kino:
  - [Kino.Input](https://kino.hexdocs.pm/Kino.Input.html)
  - [Kino.Layout](https://kino.hexdocs.pm/Kino.Layout.html)
  - [Kino.SmartCell](https://kino.hexdocs.pm/Kino.SmartCell.html)
- [Linguist PR #5672](https://github.com/github-linguist/linguist/pull/5672)

**Active Markdown**
- [reference](https://activemarkdown.org/reference.html)
- [repository](https://github.com/alecperkins/active-markdown)
- [npm](https://www.npmjs.com/package/active-markdown)
