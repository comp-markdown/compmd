# compmd

The CompMD processor, in Rust: it parses, validates and expands documents written in CompMD (Component Markdown), as the [specification](https://github.com/comp-markdown/compmd/blob/main/compmd-spec-v1.md) describes (version 1, draft).

**Status: in development.** This version has no API yet. The processor will implement stages 1 to 4b of the spec (§11) and its CEL profile (§8.2), do no I/O of its own, give every expanded node its origin (§12.3), and pass the [conformance set](https://github.com/comp-markdown/compmd/tree/main/conformance).

## Commands

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## License

[MIT](https://github.com/comp-markdown/compmd/blob/main/LICENSE-CODE). Copyright © 2026 Hakan Serce.
