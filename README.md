<h1>
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="branding/compmd-dark.svg">
    <img src="branding/compmd-light.svg" alt="CompMD" width="300">
  </picture>
</h1>

CompMD (Component Markdown) is a Markdown flavor that adds components, containers, slots and layout to CommonMark and GFM, and stays readable in any plain Markdown renderer. Files end in `.comp.md`.

**[The website](https://compmd.dev/)** shows CompMD live, with [the playground](https://compmd.dev/playground/) and [the spec](https://compmd.dev/spec/) as a page.

**Status: early draft.** CompMD is at an early stage. Until version 1 is released, the spec and the packages will change in ways that break documents and code written for an earlier draft; the [changelog](spec/CHANGELOG.md) says when one does.

## The repository's layout

Each top-level folder holds one kind of thing.

### `spec/`: the format

| File | What it holds |
|---|---|
| [compmd-spec-v1.md](spec/compmd-spec-v1.md) | The specification, version 1 (draft) |
| [compmd-registry-v1.md](spec/compmd-registry-v1.md) | *CompMD packs on a registry*, the spec's companion for pack authors and hosts: what a pack on npm must be, how a host reads one, library packs |
| [CHANGELOG.md](spec/CHANGELOG.md) | Every change to the spec and its companion, newest first |
| [llms.txt](spec/llms.txt) | The AI authoring guide: what an AI agent reads before writing CompMD (measured below) |
| [examples/complete/](spec/examples/complete/release.comp.md) | A complete document that uses every construct, with its data files |

### `crates/`: the Rust crates

| Folder | What it holds |
|---|---|
| [processor/](crates/processor/README.md) | `compmd`: the processor, which parses, validates and expands documents (in development) |
| [host/](crates/host/README.md) | `compmd-host`: what every host shares: the bundle it builds, the store of packs it gets later, and `respond`, the serving contract |
| [pack-types/](crates/pack-types/README.md) | `compmd-pack`: the names packs and their files are known by (spec §6.4), each checked once, when it is made |
| [components/](crates/components/README.md) | `compmd-components`: the component reader, what a JSX, TSX or Svelte component takes, read from its source with oxc, for W104 |
| [project/](crates/project/README.md) | `compmd-project`: reading a project as its hosts do: a document's root, its configuration, its folder and installed packs, and their integrity from the lock file |
| [lsp/](crates/lsp/README.md) | `compmd-lsp`: the language server, one binary an editor speaks LSP with over stdio; each open document is processed as the writer types, with the project's configuration and packs |
| [macros/](crates/macros/README.md) | `compmd-macros`: the processor's own macros; `#[library]` writes each expression function once, with its documentation, examples and overloads |
| [tauri/](crates/tauri/README.md) | `tauri-plugin-compmd`: CompMD in a Tauri app; its example app, the plugin's test in a real webview, is in `examples/tauri-app/` |
| [markdown-rs/](https://github.com/comp-markdown/markdown-rs/tree/compmd) | The processor's parser: a fork of markdown-rs with CompMD's syntax, a git submodule (clone with `--recurse-submodules`) |

### `grammars/`: CompMD's grammars for editors

A git submodule, a repository of its own (clone with `--recurse-submodules`), as editors and their stores take grammars ([editor support](docs/editor-support.md), decision 10).

| Folder | What it holds |
|---|---|
| [compmd-grammars/](https://github.com/comp-markdown/compmd-grammars) | `tree-sitter-compmd`, the tree-sitter grammar for Neovim, Helix, Zed and Emacs; `@compmd/textmate`, the TextMate grammar for VS Code and Shiki; and the syntax and tests they share |

### `npm-packages/`: the npm packages

`wasm/` and `host-js/` are a Rust crate and an npm package both: each compiles a crate of `crates/` to WebAssembly and publishes it with its JavaScript.

| Folder | What it holds |
|---|---|
| [wasm/](npm-packages/wasm/README.md) | `@compmd/processor`: the processor as WebAssembly for the browser, one typed call from text to trees |
| [render/](npm-packages/render/README.md) | `@compmd/render`: an expanded tree as a live page, with the host contract's bindings; the reference rendering of the built-ins |
| [sandbox/](npm-packages/sandbox/README.md) | `@compmd/sandbox`: runs components at the spec's trust levels, in strict and permissive frames or trusted in the page, so no host writes a sandbox policy itself |
| [host-js/](npm-packages/host-js/README.md) | `@compmd/host`: the serving contract over the Fetch API, `respond(request)`, for workers, Node and dev servers; what to serve is `compmd-host`'s decision, compiled to WebAssembly |
| [registry/](npm-packages/registry/README.md) | `@compmd/registry`: packs from an npm registry, resolved as the processor asks, the host's policy applied before any download, each tarball checked against its integrity, cached; a proxy for hosts that keep readers private |
| [vite/](npm-packages/vite/README.md) | `@compmd/vite`: a Vite plugin that reads `compmd.config.json`, gives the processor the app's packs, serves the sandbox and the packs' modules, and builds the app's own components with it |
| [pack/](npm-packages/pack/README.md) | `@compmd/pack`: `compmd-pack` builds, checks, tests and serves a pack's test page, so packs copy none of it; helpers for components that draw asynchronously, and a kit to test them |
| [create-pack/](npm-packages/create-pack/README.md) | `@compmd/create-pack`: `npm create @compmd/pack` writes a pack on `@compmd/pack`, its component one built module, with a test page in the sandbox |
| [devtools/](npm-packages/devtools/README.md) | `@compmd/devtools`: the Developer Tools as a part any page embeds: a document's text and its page side by side, the views of what the processor makes of it, and the sandbox's frames, a console and the messages between them |
| [blog/](npm-packages/blog/README.md) | `@compmd/blog`: a folder of CompMD documents made a static site, with a page per post |
| [create-blog/](npm-packages/create-blog/README.md) | `@compmd/create-blog`: `npm create @compmd/blog` writes a blog on `@compmd/blog` |

### `compmd-packs/`: CompMD's own packs

| Folder | What it holds |
|---|---|
| [mermaid/](compmd-packs/mermaid/README.md) | `@comp-markdown/mermaid`: Mermaid diagrams, drawn from ` ```mermaid ` blocks and from the document's data |
| [wireframe/](compmd-packs/wireframe/README.md) | `@comp-markdown/wireframe`: wireframes, drawn from ```` ```wireframe ```` blocks |

### The other folders

| Folder | What it holds |
|---|---|
| [conformance/](conformance/README.md) | Test cases any implementation can run (spec §13.2) |
| [guides/](guides/README.md) | The CompMD guides: the Author's Guide, for writing documents, and the Developer's Guide, for building with CompMD, from a first page to a published pack; their examples are tested |
| [website/](website/README.md) | [The CompMD website](https://compmd.dev/): the landing page, with the playground and the spec, published on GitHub Pages. [The playground](website/playground/README.md) is in `website/playground/`: the site's page that embeds the Developer Tools, with a menu of documents and the host's stylesheets to try, which works offline once loaded |
| [docs/](docs/README.md) | The design documents, [indexed by kind](docs/README.md), and how the repository is run: [releases and versions](docs/releases.md), [CI on GitHub](docs/ci.md); the spec [compared with MDX, Markdoc, Nuxt MDC and others](docs/existing-specs-comparison.md) |
| [branding/](branding/README.md) | The CompMD logo: the mark and the name, light, dark and in one color, and the favicon |
| `tools/` | The repository's own tools: `linux-ci/`, a container like CI's Linux runner (`cargo make linux`), and `spec-revision/`, which cuts the spec's revisions (`cargo make spec-revision`) |
| `.agentic-task-manager/` | The task list and the code reviews |

At the root: [RELEASING.md](RELEASING.md), how to release the packages, the crates and the spec, step by step; [AGENTS.md](AGENTS.md), the instructions for agents working here; the license files; and the Cargo, pnpm and lint configuration.

The spec is a document format of its own and doesn't depend on any application.

## Publishing version 1

*Proposed; to be confirmed.* Version 1 is published when every item below is done. Publishing freezes version 1: later changes are minor versions that only add (§2.2).

**When.** Publishing needs an outside reason: another implementer, users asking, or a partner. Until then the spec stays a draft in this repository, with its own changelog and conformance set.

**Checklist** (status as of 2026-10-27):

| | Item | Status |
|---|---|---|
| 1 | No open issues: every item in §17 is decided or deferred with a reason | To do: record keys are open |
| 2 | The tree schema (§12.4), and every expected tree in the conformance set fits it | Done |
| 3 | The changelog, with a version line in the spec | Done |
| 4 | The conformance set covers every construct and every diagnostic code | Done: 188 cases, all 57 codes |
| 5 | An implementation passes the whole set, including the `js` case (the plain binding) | Partly: the prototype passes all 172 cases it can run; the `js` case needs a plain-JavaScript runner, and the `origins` case an implementation of the `origins` feature |
| 6 | Plain renderers: every case degrades cleanly in a CommonMark and a GFM renderer | Done (the prototype's degradation report: 174/174 each) |
| 7 | The AI authoring guide, `llms.txt`, measured with an AI writing documents | Done (see below) |
| 8 | A review of the whole spec, start to end, with its editor | To do |
| 9 | License files: [`LICENSE`](LICENSE) (CC BY 4.0) for the text, [`LICENSE-CODE`](LICENSE-CODE) (MIT) for the conformance set and code | Done |
| 10 | The media type `text/markdown; variant=compmd` submitted to IANA's Markdown Variants registry (RFC 7764) | To do, at publication |
| 11 | The spec, conformance set and examples in `comp-markdown/compmd`, tagged `spec-v1.0` ([releases](docs/releases.md)); applications that implement it use it pinned (for example, as a git submodule) | Partly: the repository exists as a public draft; tagged `spec-v1.0` at publication |

## The AI authoring guide

AI models know MDX, Markdoc and MDC far better than CompMD, so [llms.txt](spec/llms.txt) gives an agent everything it needs to write CompMD in one file: ten rules, the syntax by example, the `md.` namespace and the CEL profile, data, inputs, layout, definitions, the habits from other formats that go wrong here, and every diagnostic code with its fix. It summarizes the spec and is checked against it: every example in it passes a processor with no errors.

**How it was measured (2026-10-23).**

- **Tasks:** 12 short descriptions, written for the measurement: six documents (release notes, a team page, a sales report, a tool's overview page, an event invitation, a changelog that defines its own component) and six app screens (a sales dashboard with two inputs, a task board, a settings form, an overdue-tasks list with a sticky header, a searchable team directory, a screen with a local `tsx` component). Each task names what it may use: a pack `@acme/ui` (`card`, `callout`, `badge`, `chart`, `table`, with their params and slots) and three data files (`team.yaml`, `sales.csv`, `tasks.csv`, with their fields).
- **Writers:** Claude Sonnet and Claude Haiku, one fresh agent per task, given only the guide and the task, with no other files and no web.
- **Baseline:** the same 24 runs given a one-paragraph description of CompMD instead of the guide.
- **Checking:** each document went through a prototype processor (stages 1 to 4b) with the pack and the data files. A document is valid when it has no `F` diagnostic. A writer whose document had diagnostics got them back once, as the processor reports them, and rewrote it.
- **Targets**, agreed before measuring: at least 11 of 12 valid on the first try, and 12 of 12 after one round of diagnostics, for each model.

**Results.**

| Writer | Valid, first try | Valid after one round | Warnings |
| --- | --- | --- | --- |
| Sonnet, with the guide | 12 of 12 | 12 of 12 | none |
| Haiku, with the guide | 12 of 12 | 12 of 12 | one W106, fixed in one round |
| Sonnet, baseline | 0 of 12 | not measured | |
| Haiku, baseline | 0 of 12 | not measured | |

- **Both targets are met for both models.** The documents use the hard parts, not workarounds: filtered and sorted `each` with `empty` slots, inputs bound to input components, `value` with formatting functions, a definition styled from its fence, breakpoints, a host component with a signature fence.
- **The one mistake with the guide** was `{{md.date(md.doc.date, "long")}}`: a function call in an interpolation, which shows as written (W106). Rule 5 now names this case with its fix.
- **Without the guide, models write other formats' syntax.** The most frequent baseline diagnostics: components used without importing the pack (F002, 48), guessed params such as `each {items=…}`, `if {test=…}` and inputs with `label=` (F008, 40), malformed component lines such as bare `:::` closers and `::: component name` definitions (F016, 25), and computed `{{ }}` (W106, 27).
- **CompMD's syntax wasn't the cause** of any failure with the guide. Without it, failures come from what a model can't guess (names, params, the definition fence), not from syntax that misleads it.
- **Limits:** one run per task and model, tasks written by the guide's author, and the prototype as the only processor. A later measurement should use tasks from real documents and more runs.

## Checks and CI

`cargo make ci` runs every check of the repository: formatting, clippy with warnings as errors, the tests with the conformance set, the crates' API docs with no broken intra-doc link, the WebAssembly build, cargo-deny, line coverage of at least 99%, and every TypeScript package built, linted and tested. CI on GitHub runs the same command on every push to `main` and every pull request ([how](docs/ci.md)); the browser benches stay local.

It needs the tools at the versions CI uses: the Rust toolchain `rust-toolchain.toml` names, with `llvm-tools-preview`; [cargo-make](https://github.com/sagiegurari/cargo-make), [cargo-deny](https://github.com/EmbarkStudios/cargo-deny), [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov) and `wasm-bindgen-cli` at the versions [`tools/versions.yml`](tools/versions.yml) names; Node and pnpm at those `package.json`'s `devEngines` names.

```sh
git submodule update --init
pnpm install
cargo make ci
```

The site's check isn't part of `cargo make ci`: the Pages workflow builds the site and checks it in Chrome before it deploys. After a change to `website/`, `guides/` or the spec, `cargo make site` runs that build and check too.

The links in the published packages' READMEs and API docs, which npmjs.com, crates.io and docs.rs show away from the repository, are checked with [lychee](https://github.com/lycheeverse/lychee) 0.24.2: `cargo make links` offline, against the files here, and CI with the network on every push to `main` ([how](docs/ci.md)). A link there to the repository is absolute, to its file on GitHub or its page on compmd.dev.

Some failures show only on CI's Linux runner, such as a page laid out a pixel wider with its fonts. `cargo make linux` runs the checks on a commit in a container like the runner, with Docker: `cargo make linux` all of them, `cargo make linux site` the site's build and check, `cargo make linux --ref <commit> …` another commit ([how](docs/ci.md)).

## License

Copyright © 2026 Hakan Serce.

- The specification text: [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/), in [`LICENSE`](LICENSE).
- The conformance set and any code: [MIT](https://opensource.org/license/mit), in [`LICENSE-CODE`](LICENSE-CODE).
