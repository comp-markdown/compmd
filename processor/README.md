# compmd

The CompMD processor, in Rust: it parses, validates and expands documents written in CompMD (Component Markdown), as the [specification](https://github.com/comp-markdown/compmd/blob/main/compmd-spec-v1.md) describes (version 1, draft).

**Status: in development.** The processor implements stages 1 to 4b of the spec (§11) and its CEL profile (§8.2), does no I/O of its own, and passes every case of the [conformance set](https://github.com/comp-markdown/compmd/tree/main/conformance) that needs no host language: it is a "compmd/1 processor with remote data and origins" (§13.1). Running host-language components is a renderer's job; the processor expands their invocations to nodes for it.

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
- Stage 4b, `expand`: makes the expanded tree (§12.3), with every node's origin when asked, and checks its layout.
- `reload`, from the loaded or expanded state: loads (and expands) again for another render context or newly fetched data, without running stages 1 to 3 again.
- `edit`, from the expanded state: the processor after a `compmd::TextEdit` of its text (a range in code points and its replacement), with everything it was given before. Every stage only redoes the top-level blocks the edit touches and moves the rest ([incremental processing](docs/incremental.md)); an edit to the frontmatter or to a component definition redoes the stages after parsing on the whole document.
- `splice`, from the expanded state: what the last edit changed in the expanded tree (`processor::Splice`), for a host that keeps its own copy of the tree.
- `host_components`, from the expanded state: each host-language component the expanded tree can name, with its language, runtime and code (`processor::HostComponent`), for a renderer that runs them: the tree names a component but doesn't hold its code. A name two documents give different components is the processed document's, or else the first loaded.

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

- A host that holds everything in memory can use maps: `HashMap<String, String>` is `Files`, `HashMap<String, Vec<host::LoadedRelease>>` is `Packs`, `HashMap<String, host::Fetched>` is `Fetch` and `HashMap<String, serde_json::Value>` is `Data`. `compmd::context::parse_time` gives a render time from RFC 3339 text. The WebAssembly build, [`@compmd/processor`](../wasm/README.md), runs the stages this way on a JSON request.
- Expressions follow the CEL profile with the `md.` functions: each is type-checked against the variables' declared types before it runs, and runs within a work budget. Formatting uses [ICU4X](https://github.com/unicode-org/icu4x) with all its compiled locale data, and time zone offsets come from the host's `Time`.
- Parsing alone is the first two calls:

```rust
use compmd::{ParseOptions, Processor};

let parsed = Processor::new("a.comp.md", "::: card {title=\"Setup\"}\nInstall it.\n::: /card\n")
    .parse(&ParseOptions::default());
let tree = serde_json::to_value(parsed.tree())?; // the JSON of §12.4
```

It isn't published yet: it depends on the fork of markdown-rs by path, and is published once the fork is, as `compmd-markdown`.

## Commands

The parser is a fork of markdown-rs in the submodule `markdown-rs/`; fetch it once after cloning. `cargo make ci` (with [cargo-make](https://github.com/sagiegurari/cargo-make)) runs every check: formatting, clippy with warnings as errors, the tests with the conformance set, [cargo-deny](https://github.com/EmbarkStudios/cargo-deny), and line coverage of at least 99% with [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov).

```sh
git submodule update --init
cargo make ci
```

## License

[MIT](https://github.com/comp-markdown/compmd/blob/main/LICENSE-CODE). Copyright © 2026 Hakan Serce.
