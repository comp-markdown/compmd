# compmd

The [CompMD](https://compmd.dev/) processor, in Rust: it parses, validates and expands documents written in CompMD (Component Markdown), as the [specification](https://github.com/comp-markdown/compmd/blob/main/spec/compmd-spec-v1.md) describes. It implements the spec's version **1.0-draft.20** (`compmd::SPEC_VERSION`); the spec is versioned on its own, apart from this crate.

**Website:** [compmd.dev](https://compmd.dev/), with the spec and a playground to try CompMD in.

**Status: in development.** The processor implements stages 1 to 4b of the spec (§11) and its CEL profile (§8.2), does no I/O of its own, and passes every case of the [conformance set](https://github.com/comp-markdown/compmd/tree/main/conformance) that needs no host language: it is a "compmd/1 processor with remote data and origins" (§13.1). Running host-language components is a renderer's job; the processor expands their invocations to nodes for it.

**Versions:**
- **0.11.0:** the spec's draft `1.0-draft.20` ([its changelog](https://github.com/comp-markdown/compmd/blob/main/spec/CHANGELOG.md), drafts 19 and 20), which `SPEC_VERSION` names: a release whose `package.json` isn't the pack and version its import names is F026; an expansion does up to 1,000,000 iterations of `each` and template slots and invocations (`expand::WORK`, `Options::work`), then stops with F034, examples counted with their document; blocks and invocations nest up to 100 deep, F038 (new) beyond; W109 for an interpolation anywhere before a URL's scheme ends; the checker knows a date (`cel::Type::Date`); `host_components` lists the components the host implements (a signature fence alone in an ambient pack, §7.1) with their pack's trust and no language or code, and every host component names its `pack`. Expressions outside a document: `compmd::cel` and `compmd::md` are public, to parse, check, evaluate and select an expression as a document's are (`Program::parse`, `check` with `md::Signatures`, `evaluate` with `md::Md`, `Error::diagnostic`, `Program::selections`). Each stage is a `tracing` span (the crate installs no subscriber). Bounded work on hostile input: CEL charges a string's or list's cost before building it, and the name suggestion is bounded. A path with `\`, `:` or NUL is F026, and a data `from: @scope/pack/file` reads only a pack the document imports, in the release it took. Diagnostics of a document with CR line endings, and a CSV's records, have their right lines; a number param `inf`, `nan` or `1e999` is F009; examples share the document's parse cache. Breaking, for documents: one past the work limit or nested deeper than 100 stops (F034, F038); a URL whose scheme an interpolation finishes renders its text without the link; a pack file read through a pack the document doesn't import, and a release that isn't the pack it was imported as, are F026. Breaking, for hosts: a document whose path isn't a `/`-separated path in its root (`./a.comp.md`, `C:\…`, empty) has F026 for every relative path it writes. Breaking, for Rust code: `HostComponent::language` is `Option<Language>`, `None` for a component the host implements; `HostComponent` (`pack`) and `expand::Options` (`work`) have new public fields, so code that builds them with struct literals adds them; `diagnostic::Code` has `F038`.
- **0.10.0:** the spec's draft `1.0-draft.18` ([its changelog](https://github.com/comp-markdown/compmd/blob/main/spec/CHANGELOG.md)), which `SPEC_VERSION` names: ten common params for a box's type (`font-size`, `font-weight`, `font-family`, `color`, `text-align` and the others), with W126 for a `color` and a `background` that contrast below 4.5:1; fonts of a document's own, `md.fonts`, and `fonts` in a pack's exports and imports, which the expanded root lists in `fonts`; gradients in `background`, checked part by part, with logical directions and angles; `grid-auto-rows` on `box` and `grid`; `object-fit` and `object-position`, how a host-language component's drawing sits in its box, and an image box, a `box` around one image, which they crop (W125 on any other box); a signature fence's param list as its box's defaults, F036 now for a component whose definition fence and signature fence both style its box; `stretch-link` (W124), `shadow`, `rotate` and `border`'s logical sides; in a definition's body, `md.fallback`, `md.slots.<name>`, interpolation in an image's alt, and a destination from the caller resolved against the caller; an inline invocation of a `compmd` component as one box; a definition's links and images written relative to the processed document, a pack's file by its place with `pack: true`; the expanded root's `lang` and `dir`; W119 only for a box whose blocks are all placed. After an edit between a component's definition and its signature fence, the fence's params move with it, so an edited processor gives what a new one does. Breaking, for documents: a signature that declares a common param (the type params, `object-fit`, `object-position`, `stretch-link`, `shadow`, `rotate` and `border`'s sides among them), `lang` or `dir` is F013, the param dropped; a box around one image with `object-fit` or `object-position` now fits its image, and any other box with them that isn't a component's has W125; an inline definition fenced with params now applies them. Breaking, for Rust code: `expanded::Root` (`lang`, `dir`, `fonts`), `tree::Link` and `tree::Image` (`pack`, and the image's `alt_parts`) have new public fields, so code that builds them with struct literals adds them.
- **0.9.0:** the spec's draft `1.0-draft.17`, which `SPEC_VERSION` names: `AllowedOrigins`, a host's listed origins or every origin, for documents it publishes itself (§8.6, §15), which `validate` takes as a list too. The host components are only those the document uses (invokes or draws a code block with, in any branch, through its `compmd` definitions), no longer every one an ambient pack gives.
- **0.8.0:** the spec's draft `1.0-draft.16` ([its changelog](https://github.com/comp-markdown/compmd/blob/main/spec/CHANGELOG.md), drafts 13 to 16), which `SPEC_VERSION`, new, names: TOML frontmatter between `+++` lines; printing, a signature's `print: fallback` (W122); examples, ` ```compmd ` blocks that render as documents of their own (W123); `render=source|output|both` on a block a code renderer draws; `plain::text`, a document's text as a reader reads it. On `compmd-markdown` 0.2.0. Breaking, for documents only: an unquoted short-form default followed by text is F011, a `+++` block at the start is the frontmatter, and a signature that declares `class` or `id`, or a code renderer's that declares `alt` or `render`, is F013.
- **0.7.1:** the same as 0.7.0; its homepage and links name the website's new address, [compmd.dev](https://compmd.dev/).
- **0.7.0:** the spec's draft as of 2026-11-12 ([its changelog](https://github.com/comp-markdown/compmd/blob/main/spec/CHANGELOG.md)); the host API names packs, releases, places and integrities with `compmd-pack`'s types, re-exported as `compmd::pack`; a pack's source component's `HostComponent` names its `place` too, so a host that compiled it ahead (spec §6.4) gives the renderer the compiled module.
- **0.6.0:** `HostComponent`, `Module` and `LibraryModule` serialize to the JSON `@compmd/sandbox` reads, so a host passes them to its page as they are.
- **0.5.0:** the same processor as 0.4.0, released with the other packages; the first crate the release workflow publishes itself.
- **0.4.0:** the spec's draft as of 2026-11-03 ([its changelog](https://github.com/comp-markdown/compmd/blob/main/spec/CHANGELOG.md)): pending packs and data (`componentPending`, with no F026), strict and lenient mode, `cache_keys`, `evaluate`, slots named with `-`, and the diagnostics 0.3.0 missed.
- **0.3.0:** the first version on crates.io.

The stages are the states of a `compmd::Processor`: each stage takes the processor and gives it back in its next state, so they can only run in order, and each state gives what its stage made and the diagnostics so far (§10.5). What the host provides comes through the small traits of `compmd::host`, each stage taking only the ones it uses: `Files` (the document's root folder), `Packs`, `Fetch` (remote data), `Data` (`host:` data) and `Time` (time zones). The processor owns what it is given, so a host can keep it as long as it likes; a host that needs its files or data too passes an `Rc` and keeps a clone of it.

- Stage 1, `parse`: the document tree (§12).
- Stage 2, `resolve`:
  - Loads a document's imports from the host's `Files` and `Packs`, with `@compmd/core` built in and the host's ambient packs (`compmd::Ambient`).
  - Binds every component name.
- Stage 3, `validate`:
  - Checks invocations, slots, interpolation paths, the types of expressions, bound inputs and data sources.
  - Records what it refused, which the later stages leave out.
- Stage 4a, `load`:
  - Gives the inputs their values for a render context.
  - Loads every data import, typed by its schema, from the root folder's files, packs, `https:` URLs (`Fetch`) and `host:` sources (`Data`).
  - Before it, the validated processor's `sources` gives the URLs and host sources a render needs (`host::Needed`, each once), so the host can fetch them first. The expanded processor's `sources` gives them for the context it keeps, after an edit to the frontmatter.
  - `sources` also gives the packs the document and the packs it loaded import that the host's `Packs` has no matching release of (`host::NeededPack`: the pack, its range as written, its pinned `integrity`), leaving out those the policy refuses, for a host that gets packs from a registry to fetch and resolve again with. `host::newest_matching` picks a version for a range as resolving does, and a release the host found doesn't match its integrity is given with `broken`, which makes an import of it F035.
- Stage 4b, `expand`: makes the expanded tree (§12.3), with every node's origin when asked, and checks its layout.
  - Its options' `mode` is lenient (the default) or strict (§10.1). Both report the same diagnostics and make the same tree; `failed()` says whether strict mode failed on an `F` diagnostic.
  - `cache_keys()` gives each data import's final URL or path, its inputs filled in, which a host keys its cached copy by (§8.6).
- `reload`, from the loaded or expanded state: loads (and expands) again for another render context or newly fetched data, without running stages 1 to 3 again.
- `edit`, from the expanded state: the processor after a `compmd::TextEdit` of its text (a range in code points and its replacement), with everything it was given before. Every stage only redoes the top-level blocks the edit touches and moves the rest ([incremental processing](https://github.com/comp-markdown/compmd/blob/main/crates/processor/docs/incremental.md)); an edit to the frontmatter or to a component definition redoes the stages after parsing on the whole document.
- `frontmatter`, from the expanded state: what the document's frontmatter gives the render (`frontmatter::Frontmatter`): its own fields (`md.doc`), its settings, each data import with what was loaded, and its inputs, for a tool that shows them.
- `components`, from the expanded state: the components the document can name (`components::Components`), grouped by the file that defines them, each with its kind, its signature, the range of every use and, for its own, of its definition; and the names it uses that stand for none, for a tool that lists them.
- `rendered_blocks`, from the expanded state: each code block a code renderer renders (`rendered_blocks::RenderedBlock`), with its language, its renderer and where its text is, so an editor hands the requests inside it to the writer's language server for that language.
- `hint_at`, from the expanded state: what is at a place in the text, and what it takes (`hints::PositionalHint`): a component's name, a param's key or value, a param list, a slot, a step of an interpolation's or an expression's path, a function, a frontmatter key or a type name; with what may be written there and the diagnostics there. It is a lookup, so an editor asks on every move of the caret, and it reads the caret's line again, so half-typed text has a hint too ([its design](https://github.com/comp-markdown/compmd/blob/main/crates/processor/docs/positional-hints.md)).
- `evaluate`, from the expanded state: a CEL expression's value in the scope the document's body sees at its top level (`md.doc`, `md.data`, `md.input` with this render's values, the `md.` functions at its time), as JSON, or its F024 placed in the expression. It is checked against the document's types and evaluated within the work budget, as an expression in the document is, and changes nothing: for a developer's console.
- `splice`, from the expanded state: what the last edit changed in the expanded tree (`processor::Splice`), for a host that keeps its own copy of the tree.
- `host_components`, from the expanded state: each host-language component the expanded tree can name, with its language, runtime and code (`processor::HostComponent`), for a renderer that runs them: the tree names a component but doesn't hold its code. A component the host implements (a signature fence alone in an ambient pack, spec §7.1) is listed too, with its pack's trust and no language or code: the host's own code runs it, at the trusted level only. A name two documents give different components is the processed document's, or else the first loaded.

Apart from any document, `functions::all()` is the catalog of the functions an expression can call (spec §8.2): the `md.` functions, CEL's functions and methods, and the macros, each with its signatures (params with names and types, the result), a description, examples with their results and the spec's section. It serializes to JSON, for an editor's completion and hover, a language server or a reference page; the positional hints read it too.

```rust
use compmd::{Ambient, ParseOptions, Processor, TextEdit};

let processor = Processor::new("notes.comp.md", text)
    .parse(&ParseOptions::default())
    .resolve(files, packs, &Ambient::default())
    .validate(&allowed_origins)
    .load(context, fetch, data)
    .expand(&Default::default());
let tree = processor.expanded(); // and processor.diagnostics()
let processor = processor.reload(other_context, fetch, data); // another input value, the same document
let processor = processor.edit(&TextEdit { start: 6, end: 11, text: "there".into() }); // the text changed
let splice = processor.splice(); // nodes `from..to` became `splice.nodes`
```

- A host that holds everything in memory can use maps: `HashMap<String, String>` is `Files`, `HashMap<String, Vec<host::LoadedRelease>>` is `Packs`, `HashMap<String, host::Fetched>` is `Fetch` and `HashMap<String, serde_json::Value>` is `Data`. `compmd::context::parse_time` gives a render time from RFC 3339 text. The WebAssembly build, [`@compmd/processor`](https://github.com/comp-markdown/compmd/blob/main/npm-packages/wasm/README.md), runs the stages this way on a JSON request.
- **What a param takes** is `compmd::values` ([its design](https://github.com/comp-markdown/compmd/blob/main/crates/processor/docs/param-values.md)): one `Kind` per kind of value (its tokens, forms and examples), which checks a value, says in an F009 what the param takes, and suggests a misspelt token. `values::vocabulary()` is the fixed vocabulary every document can write, the common params, the attributes and `box`'s params, each with its kind; `conformance/params.json` is its JSON, for the renderer and editors. A diagnostic whose message suggests a name or a token carries the edit as its `fix`.
- `compmd::plain::html` gives what a plain CommonMark and GFM renderer shows of a document (spec §14): markdown-rs with CompMD's syntax off.
- Each stage is a `tracing` span, `compmd` with its `stage` (`parse`, `resolve`, `validate`, `load`, `expand`), and ends with an event of the diagnostics it found. The crate installs no subscriber: a host that wants them installs its own.
- **An expression outside a document**, such as a computed field worked out for each row of a table: `compmd::cel` parses it once (`Program::parse`), checks it against the types of what it reads for its result type (`check`, with `compmd::md::Signatures`; a date is `Type::Date`), and evaluates it for each set of values (`evaluate`, with `compmd::md::Md`), with the same values, errors and budget as in a body. `Error::diagnostic` gives a body's F024 for an error, and `Program::selections("md.doc")` each field read by dot with where its name is, for renaming. The Developer's Guide's chapter on the Rust crates shows it.
- Expressions follow the CEL profile with the `md.` functions: each is type-checked against the variables' declared types before it runs, and runs within a work budget. Formatting uses [ICU4X](https://github.com/unicode-org/icu4x) with all its compiled locale data, and time zone offsets come from the host's `Time`.
- Parsing alone is the first two calls:

```rust
use compmd::{ParseOptions, Processor};

let parsed = Processor::new("a.comp.md", "::: card {title=\"Setup\"}\nInstall it.\n::: /card\n")
    .parse(&ParseOptions::default());
let tree = serde_json::to_value(parsed.tree())?; // the JSON of §12.4
```

It is on crates.io as `compmd`, `cargo add compmd`, with the npm packages' version: each release publishes it with them (0.3.0 is the npm packages' 0.3.0). Its parser, the fork of markdown-rs in `crates/markdown-rs/`, is published first, as `compmd-markdown`, with a version of its own that moves when its code changes. The library keeps the name `markdown`, so here it is a path dependency with that version, and the published crate depends on the version.

## Where the spec leaves the choice

What this processor does where the spec says what is wrong but not what code reports it or what renders:

- **An input without a default.** Every input must have one (§8.7). It is F010 at the declaration, and the input starts with its type's empty value: `""`, `0`, `false`, `[]`, `{}`, or an enum's first member; `null` for an optional type, and for a date.
- **Content before the first slot line of a component that declares no `default` slot.** It is F006 at the content's first line. The content follows the component as normal content, with that of any undeclared slot, so nothing the author wrote is dropped.
- **A `fallback` on a local source.** `fallback` is for remote and `host:` sources (§8.6). When a local file doesn't load, it is still F026, and its fallback, if it loads, is the data instead of `null`.

## Commands

The parser is a fork of markdown-rs in the submodule `crates/markdown-rs/`; fetch it once after cloning. `cargo make ci` (with [cargo-make](https://github.com/sagiegurari/cargo-make)) runs every check: formatting, clippy with warnings as errors, the tests with the conformance set, [cargo-deny](https://github.com/EmbarkStudios/cargo-deny), line coverage of at least 99% with [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov), and the TypeScript packages built, linted and tested (which needs pnpm), the renderer's tests checking it against `params.json`.

```sh
git submodule update --init
cargo make ci
```

## License

[MIT](https://github.com/comp-markdown/compmd/blob/main/LICENSE-CODE). Copyright © 2026 Hakan Serce.
