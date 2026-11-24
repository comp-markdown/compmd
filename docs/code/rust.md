# Rust

The repository's own Rust rules. General Rust idioms apply on top of these; [AGENTS.md](../../AGENTS.md) has the overall code quality rules.

## Toolchain and lints

- **Pinned toolchain** in `rust-toolchain.toml`, with the `wasm32-unknown-unknown` target. Upgrades happen in their own commit.
- **Edition 2024**, set once in `[workspace.package]`.
- **Lints** are set in the workspace `Cargo.toml` and inherited with `[lints] workspace = true`:
  - `unsafe_code` is forbidden;
  - `clippy::all` and `clippy::pedantic` are on, with the few noisy pedantic lints allowed there, never in code;
  - warnings fail CI.
- `#[allow(...)]` in code needs a short reason next to it.

## Crates and modules

- **Module files:** `foo.rs` next to a `foo/` directory, never `foo/mod.rs`.
- **One concept per module:** each public type and each concern in a file of its own.
- **Visibility:** `pub` only for the crate's API; `pub(crate)` for the rest.
- **Types, fields and enum variants are named with nouns** for what they are, not clauses or verbs. Functions are verbs, and booleans read as a statement.

## Dependencies

- **Ask first.** A new dependency is proposed to the maintainer with the reason and the alternatives before it is added.
- Versions live in `[workspace.dependencies]`; crates use `dep.workspace = true`, adding the features they need. The Tauri plugin, outside the workspace, keeps its own.
- Every crate declares `rust-version`, the pinned toolchain's (`[workspace.package]`), and a published crate an `include` list: its sources, its README and its licence, never its tests and benchmarks.
- cargo-deny must pass: licences, advisories, bans.

## Command-line tools

- Every Rust command-line tool parses its arguments with `clap` using the derive API (`#[derive(Parser)]`, subcommands as an enum).

## Errors

- **Library crates** define typed error enums with `thiserror`, one per crate or per area, with variants callers can act on.
- **Binaries** may use `anyhow` to add context at the top level.
- **No `unwrap`** outside tests. **`expect`** only for a proven invariant, with a message saying why it can't fail.
- Errors never contain a document's content, so they are safe to log.

## Logging

- Libraries emit structured events and spans with `tracing`. Only binaries install a subscriber.
- Never log a document's content; log positions, counts and timings.

## Documentation

- Every public item gets a `///` doc comment: what it does, and its arguments where not obvious.
- Public functions that can fail have an `# Errors` section; key APIs have a runnable `# Examples` doctest.
- Each crate's `lib.rs` starts with a `//!` summary pointing to its spec sections and designs.
- No other comments unless the code can't say it: a non-obvious reason, an invariant or a link to a decision.

## Tests

- **Unit tests stay with the code they test,** in a `#[cfg(test)] mod tests` at the bottom of its file, while the file reads well as a whole.
- **When the tests hurt the code's readability, they move out** to a file of their own, `foo/tests.rs`, declared at the bottom of `foo.rs` with `#[cfg(test)] mod tests;`. When that file grows large, `foo/tests/` holds one file per group of behaviours.
- **Integration tests are one test binary per package,** `tests/<name>.rs` while it is one file and `tests/<name>/main.rs` once it has more, named for what they test, with a module per area and their shared helpers in `common.rs`. Each file directly in `tests/` would be a test binary of its own, linking the library again, and a helper that one of them doesn't use would be dead code in it ([why](https://matklad.github.io/2021/02/27/delete-cargo-integration-tests.html)).
- **About 400 lines is a guideline for when a file reads badly, not a threshold.** Judge how the tests affect reading the code before splitting. Test data (fixtures, tables of cases) may be longer, in files of its own.
- **Property tests** with `proptest` for algebraic code.
- **Benchmarks** use `criterion` and live in the crate's `benches/` directory.
- Test names say the behaviour: `rename_keeps_values`, not `test_rename`.
- Rust crates keep at least 99% line coverage (`cargo make coverage`).
