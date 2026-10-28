# compmd

The CompMD processor, in Rust: it parses, validates and expands documents written in CompMD (Component Markdown), as the [specification](https://github.com/comp-markdown/compmd/blob/main/compmd-spec-v1.md) describes (version 1, draft).

**Status: in development.** The processor will implement stages 1 to 4b of the spec (§11) and its CEL profile (§8.2), do no I/O of its own, give every expanded node its origin (§12.3), and pass the [conformance set](https://github.com/comp-markdown/compmd/tree/main/conformance). Stage 1, parsing, is done: `compmd::parse` gives the document tree (§12) and the parse diagnostics (§10.5), and passes every parse-only case of the conformance set.

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
