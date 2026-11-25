# CompMD

Component Markdown: a Markdown flavor with components, containers, slots and layout, readable in any plain Markdown renderer. This repository holds the spec, its processor (Rust), the renderer, the sandbox, the host and registry packages, packs, the guides and the website. The [README](README.md) lists every folder.

## Where things are

- `spec/`: the spec (`compmd-spec-v1.md`) and its companion (`compmd-registry-v1.md`). `spec/CHANGELOG.md` records every change to them, under `### Unreleased` until a revision is cut (`cargo make spec-revision`, [docs/releases.md](docs/releases.md)).
- `docs/README.md`: the design documents, each leading with its status and decisions. Read the designs a change touches before changing code.
- `docs/code/`: how code is written here: [Rust](docs/code/rust.md), [Svelte](docs/code/svelte.md), [Playwright](docs/code/playwright.md).
- `conformance/`: the test cases any implementation runs (spec §13.2).
- `guides/`: the Author's Guide and the Developer's Guide; their examples are tested.
- `crates/`: the Rust crates; `npm-packages/`: the npm packages (`wasm/` and `host-js/` are a crate as well); `compmd-packs/`: CompMD's own packs; `tools/`: the repository's own tools.
- `crates/markdown-rs/`: a git submodule, the fork `comp-markdown/markdown-rs` (branch `compmd`), the processor's parser.
- `grammars/compmd-grammars/`: a git submodule, `comp-markdown/compmd-grammars`: the editors' tree-sitter and TextMate grammars and what they share. Push it before a commit here that points to it.
- `.agentic-task-manager/`: the task list and the code reviews (atm). It is public, like the rest of the repository.

## The spec comes first

- **Code does what the spec says.** When the spec has a gap or the code needs something it doesn't say, change the spec (with its changelog entry and conformance cases), never work around it in code.
- **CompMD is a format of its own** and doesn't depend on any application. Nothing here names or assumes one; examples are what people write: newsletters, posts, notes, with CompMD's own parts shown off.
- Spec changes and every push need the maintainer's go.

## Commands

- `cargo make ci`: every check CI runs: fmt, clippy (warnings are errors), coverage (99% of lines), doctests, API docs, the WebAssembly build, cargo-deny, the TypeScript packages and the scripts.
- While developing: `cargo fmt --all`, `cargo make clippy`, `cargo make test`; for the packages, `pnpm --dir <package> lint` and `pnpm --dir <package> test`.
- `cargo make site`: the website's build and check. Not part of `ci`: run it after a change to the website, the guides or the spec.
- `cargo make linux …`: a check in a container like CI's Linux runner (WebKit and fonts differ from macOS). A check that fails only on CI is reproduced there first.
- `cargo make tauri-check`: the Tauri plugin and its example app.

## How to work

- **Never assume.** If a requirement, a design point or a behaviour is not settled in the spec or the designs, ask before acting. Settle open design points in conversation, then write them into the design.
- **Decisions first** in designs and plans: the decisions at the top, new terms defined concretely, then the rest. Test a spec proposal on real example documents and list its gaps up front.
- **Verify before claiming.** Run the tests and checks; report failures as they are.
- **Stay in scope.** Do what the task asks.
- **Docs:** plain wording, one edit per change.
- **Git:** commit only when asked, never push unless asked, no `Co-Authored-By`, `Claude-Session` or other AI attribution lines in commit messages. Push the `markdown-rs` fork before a commit here that points to it.

## Tasks and reviews

Tasks and code reviews go through atm, the agentic task manager (`atm docs` lists its reference for agents). Never edit `.agentic-task-manager/` by hand.

- A task is committed, then reviewed as an atm code review (`atm review create --task <key> --range <base>..HEAD`), and done once the review is approved.
- A task whose scope isn't finished (only a design, follow-ups missing) is blocked, not in review.
- Task text, notes and review comments are public: write them as the rest of the repository is written.

## Code quality

Every change leaves the code better than it found it.

- **Simple and elegant.** KISS, DRY, YAGNI. When there is a simpler, clearer or more maintainable way to write something, use it.
- **SOLID**, especially open-closed: extend behaviour through new types and trait implementations rather than editing working code paths.
- **One concept per module:** each public type and each concern in a file of its own from the first commit.
- **Code documents itself.** Clear names and small functions instead of comments. Comment only what the code can't say (a non-obvious reason or constraint). Tests show how the code is meant to be used.
- **Public APIs get high-quality docs:** what it does, its arguments, errors and an example where useful (`///` in Rust, TSDoc in TypeScript).
- **Idiomatic code**, with the rules in `docs/code/`.

## Testing

- **Unit tests** next to the code; property tests for algebraic code.
- **Conformance cases** for every construct and every diagnostic code the spec defines.
- **Browser tests** for what runs in a page, in Chrome and WebKit.
- Test files stay readable. In TypeScript, tests are in `foo.test.ts` next to `foo.ts`, and a test file that grows large is split, one file per capability. About 400 lines is a guideline, not a threshold.

Rust crates keep at least 99% line coverage. A change is done when its tests pass and `cargo make ci` is green.
