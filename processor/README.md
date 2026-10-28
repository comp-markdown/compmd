# compmd

The CompMD processor, in Rust: it parses, validates and expands documents written in CompMD (Component Markdown), as the [specification](https://github.com/comp-markdown/compmd/blob/main/compmd-spec-v1.md) describes (version 1, draft).

**Status: in development.** The processor implements stages 1 to 4b of the spec (§11) and its CEL profile (§8.2), does no I/O of its own, and passes every case of the [conformance set](https://github.com/comp-markdown/compmd/tree/main/conformance) that needs no host language: it is a "compmd/1 processor with remote data and origins" (§13.1). Running host-language components is a renderer's job; the processor expands their invocations to nodes for it.

- Stage 1, parsing: `compmd::parse` gives the document tree (§12) and the parse diagnostics (§10.5).
- Stage 2, resolving: `compmd::resolve::resolve` loads a document's imports through the caller's `Files` and `Registry`, with `@compmd/core` built in and the caller's ambient packs, and binds every component name.
- Stage 3, validating: `compmd::validate::validate` checks invocations, slots, interpolation paths, the types of expressions, bound inputs and data sources, and gives what it refused, which the later stages leave out.
- Stage 4a, loading: `compmd::load::load` gives the inputs their values and loads every data import, from the project's files, packs, `https:` URLs (`Remote`) and `host:` sources (`HostData`), typed by its schema. `compmd::load::sources` lists the URLs and host sources a render needs, so the caller can fetch them first.
- Stage 4b, evaluating: `compmd::expand::expand` makes the expanded tree (§12.3), with every node's origin when asked, and checks its layout.
- The CEL profile: `compmd::cel` parses an expression, type-checks it against the variables' declared types, and evaluates it within a work budget, with the `md.` functions in `compmd::md`. Formatting uses [ICU4X](https://github.com/unicode-org/icu4x) with all its compiled locale data, and time zone offsets come from the caller.

```rust
use compmd::cel::{Program, Scope, Type};
use compmd::md::Signatures;

let program = Program::parse("md.plural(count, 'item', 'items')")?;
assert_eq!(program.check(&Scope::default().with("count", Type::Double), &Signatures)?, Type::String);
```

```rust
use compmd::{ParseOptions, parse};

let parsed = parse("::: card {title=\"Setup\"}\nInstall it.\n::: /card\n", &ParseOptions::default());
let tree = serde_json::to_value(&parsed.root)?; // the JSON of §12.4
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
