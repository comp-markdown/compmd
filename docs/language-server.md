# The language server

**Status: proposed (2026-11-25).** Builds decisions 5, 8 and 9 of [CompMD in editors](editor-support.md): what the language server is, what it is written in, where a document's root and packs come from, and what each of its features asks of the processor. It takes in the ideas noted for it in the inbox (IN-1). Its parts are built as tasks of their own, listed at the end.

## Decisions

1. **One native binary, `compmd-lsp`, written in Rust, on the processor crate.** It links `compmd` and `compmd-host` as they are, so it parses what the spec says with no copy of the processor, and needs nothing installed beside it: no Node, no Python. The [options](#the-executable-rust-node-or-both) compare Rust, Node and a mix. Node is never needed to run it, and a writer with Node gains nothing they would miss without it.

2. **Synchronous, on threads, with `lsp-server` and `lsp-types`** (rust-analyzer's crates), as `editor-support.md` says: a main loop reads messages and hands each document's work to its own worker thread, which owns the document's processor. There is no async runtime: network work (registry packs, decision 6) runs on a thread of its own and posts its result back as a message. A request about a document waits only for that document's worker.

3. **A document's root folder is the nearest one above it that says so; else the editor's; else its own.** The root is the folder a document's relative paths resolve in (spec §6.1: anything outside it is F026). In this order, the first that applies:
   1. the nearest folder above the document with `compmd.config.json`, or a `package.json` with a `compmd` object (the rule `@compmd/vite` uses for a page, [pack settings by page](vite-per-page-config.md), decision 1);
   2. the workspace folder the editor gives that holds the document (`workspaceFolders`). Each editor decides it: mostly a git repository's top, or a project's marker file (`.git`, `package.json`, Neovim's `root_markers`, Helix's `roots`, a VS Code workspace);
   3. the document's own folder.

   That root's configuration is the document's, as a host reads it. With none, the document has the default: every installed pack strict, no registry, no origins.

4. **The editor sees what the site will render: diagnostics come from the document's own configuration, nothing else.** The server resolves imports and loads packs exactly as a host with that configuration does. An import the site couldn't load is F026 in the editor too, never quietly fixed by what the editor can reach. What the editor can reach beyond that (decision 7) only *suggests*: completions, and code actions that add an import or install a pack. It never changes a diagnostic.

5. **Packs come from five places, in the host's order** ([hosting](hosting.md), decision 4):

   | Place | Example | How the server gets it | Diagnostics |
   |---|---|---|---|
   | **built in** | `@compmd/core` | the processor has it | always |
   | **the document's own files** | `./chart.jsx`, `./cards.comp.md`, a `component=` fence | the files under the root, read as the editor holds them (an unsaved file's text included) | always |
   | **folder packs** | `./components` in `compmd.config.json` | read from the folder, relative to the configuration's folder | as the configuration lists them |
   | **installed packs** | `@comp-markdown/mermaid` in `node_modules` | found as `@compmd/vite` finds them: the root's dependencies that are packs, their integrity from the lock file | always available, strict unless listed |
   | **registry packs** | any pack an import names, from npm or a private registry | fetched by the configuration's `registries`, checked and cached (decision 6) | only when the configuration names a registry for its scope |

   **Every source is one abstraction, the processor's own.** The processor already takes packs through one trait, `compmd::host::Packs` (a pack's releases, a pack file by its place, and `pending` for one on its way), and a document's files through `Files`. Each source implements `Packs`, and one chain puts them in the configuration's order:

   | Implementation | Source | Crate |
   |---|---|---|
   | the processor's own | built in (`@compmd/core`) | `compmd` |
   | `FolderPacks` | folder packs, by the configuration | `compmd-project` |
   | `InstalledPacks` | installed packs, by `node_modules` and the lock file | `compmd-project` |
   | `RegistryPacks` | registry packs: what the cache holds, `pending` while a fetch is on its way | `compmd-host` (H1) |
   | `HostPacks` (exists) | a built bundle and its store, for a host that serves one | `compmd-host` |
   | `ChainedPacks` | the sources above, the first that has a pack giving it | `compmd-host` |

   The document's own files are the processor's `Files`: the editor's implementation gives an open document's text as the editor holds it, and the disk's otherwise. A host composes what it has: the server chains the folder, installed and registry packs; `tauri-plugin-compmd` its bundle and store; a command line the same as the server. A new source (a host's marketplace, a workspace's packs) is one more implementation, and nothing that reads packs changes.

   **Known packs are one more method, not another abstraction.** What completion offers beyond the document (decision 7) is a pack source's listing: `Packs` gains `fn known(&self) -> Vec<KnownPack>` (each pack's name, version, description and exported components with their signatures), empty by default, so a host that doesn't suggest (a site) implements nothing. The installed and folder sources list what they hold; the registry source lists its suggested scopes.

   `compmd-project` reads the project: the configuration, the folder and installed sources, and the lock file's integrity. It sits beside `compmd-host`, so the server, a command line and `tauri-plugin-compmd` read a project as `@compmd/vite` does, with the same fixtures checking both. `@compmd/vite` keeps its TypeScript for now; using the crate as WebAssembly instead is its own question (Q3).

6. **Registry packs through a registry client in Rust, as `hosting.md` already plans (H1, option c).** The resolver's loop, ranges matched as the processor matches them, the policy before any download, the integrity checks and the tar reader move into `compmd-host`, which `@compmd/registry` then uses as WebAssembly. The server is that client's first user. It takes an HTTP client (a small blocking one, `ureq`) and a cache: a folder in the user's cache directory (`~/.cache/compmd/packs` and its platform equivalents), keyed by a release's pack, version and integrity, as `PackCache` is. The server fetches the way the configuration says (`fetch: "host"` or `"direct"`): either way, from the writer's own machine.

7. **What the editor offers beyond the document's configuration: the known packs, set by the writer.** Completion and auto-import can offer a pack the document doesn't import yet, from a list of **known packs**:
   - the installed and folder packs (no network);
   - the packs of the **suggested scopes**: `@comp-markdown` by default, CompMD's own packs, with others the writer adds. The server lists a scope's packs from its registry's search (npm's `/-/v1/search`, by the scope and the keyword `compmd-pack`, which only packs use, [packs from a registry](packs-from-a-registry.md) R6; a registry without search offers only what is installed or cached), fetches each listed pack's latest release, and reads the names and signatures its document exports;
   - a writer turns this off (`compmd.packs.network: false`), and the server then suggests only what is on the machine.

   The list is the writer's, not the document's: it lives in the editor's settings (decision 12), since which packs a writer likes to be offered is about them, not about the site. A project may add its own scopes for everyone who opens it (`compmd.config.json`'s `editor.suggest`, Q2). Choosing a known pack from a registry doesn't make the site able to load it: the code action that imports it says so with the import's F026, and offers the fix the configuration needs (Q4).

8. **One processor per open document, edited as the writer types.** A document's worker keeps its expanded processor, applies each change as a `TextEdit` (the processor's incremental processing, [incremental.md](../crates/processor/docs/incremental.md)), and publishes the diagnostics after each. A change to a file the document imports, to the configuration or to `node_modules` reloads the documents that use it (`workspace/didChangeWatchedFiles`, which the server registers for). Hints, completions and hovers are lookups in the processor's state ([positional hints](../crates/processor/docs/positional-hints.md)), never another run.

9. **Every feature is the processor's, given in LSP terms.** The server writes no CompMD logic of its own: what a name stands for, what a param takes, what a function does and what a block's language is come from the processor (`hint_at`, `components`, `rendered_blocks`, `functions`, the trees). Where a feature needs something the processor doesn't give yet, the processor gains it, for the WebAssembly package and the Developer Tools too. The features and their processor APIs are in [the features](#the-features).

10. **Templates, code actions and refactorings are the server's, so every editor has the same ones** (`editor-support.md`, decision 5). They come from the signatures the document can name, with params filled from their defaults and the values their types take first. The ideas in IN-1 are each placed in [what IN-1 asked for](#what-in-1-asked-for).

11. **A host-language component's signature is written from its code by a component reader in Rust.** Generating or updating a `signature` block (IN-1) reads a component's props: `.jsx`, `.tsx`, `.js` and `.ts` with `oxc` (a Rust parser for JavaScript and TypeScript), a `.svelte` file's `<script>` with the same parser (Svelte 5's `let { … }: Props = $props()`). It reads what the source states, as `@compmd/vite`'s analysis does (W104): a prop's name, a TypeScript type it can map, its default. A prop it can't read is left for the writer, with a comment, never guessed. The same reader can give the processor W104 for a document's own components (Q5).

12. **Settings come from the editor, in one namespace, `compmd`** (`workspace/configuration`, with `initializationOptions` for editors that only send those): the suggested scopes and the network switch (decision 7), the preview's browser (decision 13), the formatting's options, the trace level. Everything about how a document renders comes from its `compmd.config.json`, never from settings.

13. **The preview is the server's, as `editor-support.md` decision 8 says**, on `127.0.0.1` with a token: `compmd.preview` opens it, and an editor package's *toggle* (IN-1) opens it or brings its tab forward. Closing a browser tab is the browser's, so *toggle* in an editor without a browser of its own opens or focuses, and VS Code's webview closes it.

14. **Released as `editor-support.md` decision 9 says**: binaries for macOS, Linux and Windows on x64 and arm64 on each GitHub release with their SHA-256, `cargo install compmd-lsp`, and `@compmd/lsp` on npm with one package per platform (as esbuild and Biome ship theirs), so mason.nvim, `npm i -g` and editors that fetch from npm all find it.

## Terms

- **Root:** the folder a document's relative paths resolve in, and whose configuration it has (decision 3).
- **Known packs:** the packs completion and auto-import offer, beyond what the document imports (decision 7).
- **Suggested scopes:** the scopes whose packs are known packs, `@comp-markdown` by default.
- **Worker:** the thread that owns one open document's processor (decision 2).

## The executable: Rust, Node or both

What decides it: the server runs the processor, which is Rust; it must work in every editor, including those whose users don't have Node (Helix, Emacs, a plain Neovim); and some of CompMD's tooling is TypeScript today (`@compmd/vite`'s pack finding and analysis, `@compmd/registry`).

| | **Rust binary** (proposed) | **Node package** | **Rust binary, with a Node helper when there is Node** |
|---|---|---|---|
| The processor | native, the crate itself | its WebAssembly build (3.1 MB, 1 MB gzipped), slower than native and on one thread (to be measured on the incremental benchmarks) | native |
| Runs with | nothing else | Node 22 or later, on the writer's `PATH` | nothing else; more with Node |
| Installs | one file per platform; `cargo install`, npm platform packages, mason, release binaries | `npm i -g`; mason needs Node | as Rust, plus the helper's npm package |
| Packs, registry, analysis | in Rust: `compmd-project`, the registry client (H1), the component reader (`oxc`) | reuses `@compmd/vite`, `@compmd/registry` and TypeScript's compiler as they are | Rust for what it has, the helper for the rest |
| Start and memory | about 10 ms and tens of MB | Node's start, a few hundred ms, and its heap | as Rust, plus Node's when the helper starts |
| Size | tens of MB, most of it ICU4X's data for every locale, which the WebAssembly carries too (to be measured with the first build) | small, plus Node | as Rust, plus the helper |
| LSP library | `lsp-server`/`lsp-types` (rust-analyzer, Ruff), or `tower-lsp` (Biome) | `vscode-languageserver` (VS Code's own servers, YAML, Tailwind) | both |
| Two of everything? | the pack reading and the registry rules exist in Rust and in TypeScript until `@compmd/vite` and `@compmd/registry` use the Rust as WebAssembly (H1's plan) | no | yes: two runtimes, two protocols between them |

**What other servers do.** Servers for a language whose own tools are Rust or Go ship a native binary: rust-analyzer, Ruff, Biome, Taplo (TOML), gopls, marksman (Markdown, .NET native). Servers built on a TypeScript toolchain ship as Node packages: TypeScript's, ESLint's, Tailwind's, YAML's, and VS Code's own Markdown server. Editors that aren't VS Code install native binaries most easily: Helix and Emacs have no Node of their own, and mason.nvim's Node servers fail where Node isn't installed.

**Proposed: the Rust binary.** CompMD's core is Rust, so a native server has the processor at full speed with nothing else to install. The cost is the TypeScript that has to exist in Rust too: pack finding (decision 5), the registry rules (decision 6, already planned as H1) and a component reader (decision 11). Each becomes the one implementation over time, as `@compmd/processor` already is for the processor. A Node helper would bring that cost back in another form: two runtimes in every install and a protocol between them.

## The features

Each is a task of its own (see [tasks](#tasks)). LSP methods are named as the protocol does.

### The core: documents, diagnostics, settings

- `initialize` with the capabilities below; `textDocument/didOpen`, `didChange` (incremental: the edit's range becomes the processor's `TextEdit`, in UTF-16 positions as LSP sends them, converted to code points), `didClose`, `didSave`.
- `textDocument/publishDiagnostics` after each change: the processor's diagnostics (spec §10.5), each with its code (`F026`), its severity, its range, its related places (`relatedInformation`) and a link to the spec's section for its code (`codeDescription`). A diagnostic with a `fix` is also a quick fix (*code actions*, below).
- `workspace/didChangeWatchedFiles`: `compmd.config.json`, `package.json`, the lock files, `node_modules/*/package.json`, and every file an open document imports reload the documents that read them.
- `workspace/configuration` and `didChangeConfiguration`: the settings (decision 12).
- A document of another processed folder (a pack's own document, a file an import names) opens as a document of its own, with its own root.

### Semantic tokens

- `textDocument/semanticTokens/full` and `/range`, by the shared token names (`syntax/tokens.json`: `component-name`, `param-key`, `interpolation-path`, …), from the processor's parse tree, which knows more than a grammar can: a name that is a component the document can name, one that is unknown (with the `deprecated` or a custom `unknown` modifier, Q6), a param its signature declares, one it doesn't.
- `compmd-grammars`' shared test set checks the server's tokens, as it checks the two grammars (`editor-support.md`, decision 2).

### Completion, hover, signature help, go to definition

- `textDocument/completion` from `hint_at`'s completions: component names (with their signature as `detail`), params and their values, slots, frontmatter keys and their values, type names, `md.` paths and fields, functions and methods by the type of the value before the `.` (`compmd::functions::of_type`), each with its documentation from the catalog (`compmd::functions`): its signatures, its description and an example with its result (IN-1, *CEL*).
- On a ` ```compmd ` example's param list (spec §7.8): `render` (`output`, `source`, `both`) and the common params. The processor's hints give them for a code renderer's block only, since `hints/invocation.rs` builds a hint around a component and an example has none; the processor gains them (noted in CMD-24 from CMD-37).
- **Auto-import** (IN-1): a component name the document can't name yet, offered from the known packs (decision 7), with `additionalTextEdits` that add the import to the frontmatter's `md.import`, in the frontmatter's own syntax, YAML or TOML.
- `textDocument/hover`: what `hint_at`'s subject says, as Markdown: a component's origin and signature, a param's type, default and doc, a path step's type and what it is, a function's signatures, description and first example, a frontmatter key's meaning, a type's values, and the diagnostics at the place.
- `textDocument/signatureHelp`: inside an expression's call, the function's signatures with the current argument; inside a component's param list, its signature's params.
- `textDocument/definition`: an import's `from` to its file or pack document; a component's name to its definition fence, or to the file or pack that exports it; an interpolation's `md.data.<name>` to its data import.

### Blocks, symbols, folding

- `compmd/blocks` (a custom request): `rendered_blocks`, each block's language `id`, its range and the language's other names (content languages, decision 3), for an editor package to forward requests inside it to the writer's server for that language.
- `textDocument/documentSymbol`: headings, containers, leaves and definitions, nested as the document nests them.
- `textDocument/foldingRange`: the frontmatter, containers (an opener to its closer), definition fences, rendered blocks.

### Templates, code actions, refactorings (IN-1)

- **Templates** are completions with snippets (`insertTextFormat: Snippet`), at the start of a line: a container with its signature's required params as tab stops and the rest in a choice, a `row` of columns, a leaf, an inline component, a slot of the container the line is in, a component definition (`compmd`, or a host language's fence with its `signature` block), an `each` and an `if`. From the signatures the document can name, never a fixed list.
- **Code actions** (`textDocument/codeAction`), each of a kind an editor groups:
  - `quickfix`: a diagnostic's `fix`; import a known pack for an unknown name; install a pack the import names (Q4).
  - `source.organizeImports`: `md.import` sorted by source, names merged and sorted, unused names dropped, in the frontmatter's syntax.
  - `source.organizeFrontmatter.compmd`: the frontmatter's keys in a canonical order (the author's keys first, then `md`'s in the spec's order), with the common keys a document lacks offered (`title`, `md.lang`, Q7).
  - `refactor.rewrite`: the frontmatter from YAML to TOML and back (spec §2.1 reads both), keeping comments where the other syntax has them.
  - `refactor.extract`: a selection into an inline `compmd` component (a definition fence in the document and a use where the selection was), or into a component file of its own, imported; an inline component into a file of its own; a selection into a container.
  - `refactor.inline`: a `compmd` component's use as what it expands to.
  - `refactor.rewrite`: a leaf into a container and back, where its slots allow; *generate signature* and *update signature* for a host-language component (decision 11), which write or rewrite its `signature` block from its props.
  - `textDocument/rename` for a component across the document and the files that import it.
- **Formatting** (`textDocument/formatting`, `rangeFormatting`): its rules are written first, as its task says (`editor-support.md`, decision 5).
- **The preview**: the command `compmd.preview` and the code action *Open preview* (decision 13).

## What IN-1 asked for

| Idea | Where it is |
|---|---|
| templates for common containers, with params filled | templates: snippets from the signatures, required params as tab stops |
| organize imports | `source.organizeImports` |
| organize the frontmatter, and add common properties | `source.organizeFrontmatter.compmd`; which keys are common is Q7 |
| frontmatter to and from YAML and TOML | `refactor.rewrite` |
| auto-import a known pack (all of `comp-markdown`) | completion's auto-import, from the known packs; `@comp-markdown` a suggested scope by default (decision 7) |
| completion of CEL, with documentation | completion from the function catalog and the per-type listing (CMD-408, CMD-409) |
| extract an inline component from a selection | `refactor.extract` |
| extract an external component from a selection or an inline component | `refactor.extract`, into a file of its own, imported |
| generate, regenerate or update a host component's signature block | `refactor.rewrite`, with the component reader (decision 11) |
| toggle the preview | the preview's command, and each editor package's toggle (decision 13) |

## Architecture

```
crates/
  project/          compmd-project: the root, compmd.config.json, folder and installed packs, lock files' integrity
  host/             compmd-host: + the registry client and its folder cache (H1)
  lsp/              compmd-lsp: the binary
    src/main.rs        stdio, the main loop
    src/workspace.rs   roots, settings, watched files, known packs
    src/document.rs    a document's worker and its processor
    src/convert/       positions (UTF-16 ↔ code points), diagnostics, hints → LSP types
    src/features/      one module per feature: tokens, completion, hover, definition, symbols, actions, …
    src/preview/       the preview's server (decision 13)
```

- **Positions:** the processor counts code points; LSP counts UTF-16 by default. The server asks for UTF-8 or UTF-32 when the client offers them (`general.positionEncodings`), and otherwise converts with the document's line index.
- **Errors are diagnostics, never crashes:** a pack that can't be read is the processor's F026 with why; a panic in a document's worker is caught, logged (`window/logMessage`), and the document reopened.

## Questions, with proposed answers

**Q1. The crate for reading a project.** A new `compmd-project`, or part of `compmd-host`? *Proposed:* a crate of its own. `compmd-host` reads a built bundle and serves it; reading a project's sources (its configuration, `node_modules`, a lock file) is what a build and an editor do, not a host.

**Q2. A project's own suggested scopes.** Should `compmd.config.json` take an `editor` key (`{ "editor": { "suggest": ["@acme"] } }`) so everyone who opens the project is offered its scopes? *Proposed:* yes, beside the writer's settings, which add to it. It changes `@compmd/vite`'s schema, which ignores the key.

**Q3. `@compmd/vite` on `compmd-project`.** Should the Vite plugin read projects through the Rust crate as WebAssembly, as `@compmd/registry` will (H1)? *Proposed:* later, as its own task; until then both are checked against the same fixtures.

**Q4. Installing a pack from the editor.** When auto-import picks a registry pack the configuration can't load, should the code action run the package manager (`pnpm add`), or only say what to run? *Proposed:* say it, with the command in the diagnostic's message and a code action that adds the registry to `compmd.config.json` where that is the writer's intent. The server runs no package manager: it would run scripts from `package.json`.

**Q5. W104 for a document's own components.** The component reader (decision 11) could give the processor W104 for a document's own `./Chart.jsx`, not only for listed packs. *Proposed:* yes, as a later task: the processor gains an optional analysis input, and every host can give it.

**Q6. An unknown component's token.** *Proposed:* `component-name` with a modifier `unknown`, which themes can colour, beside the diagnostic.

**Q7. "Common" frontmatter keys.** *Proposed:* the keys the spec's examples and the Author's Guide use for posts and notes (`title`, `description`, `date`, `tags`, `md.lang`), offered, never added unasked.

## Tasks

The work, in order; each is a task with its own review:

1. **`compmd-project`**: the root, the configuration, folder and installed packs, lock files' integrity; checked against `@compmd/vite`'s fixtures.
2. **The server's core**: the binary, documents and their workers, incremental processing, diagnostics, settings, watched files.
3. **Semantic tokens**, checked by `compmd-grammars`' test set.
4. **Completion, hover, signature help, go to definition**, with auto-import from installed packs.
5. **`compmd/blocks`, document symbols, folding.**
6. **The registry client in Rust** (H1), then the known packs from registries (decision 7).
7. **Templates.**
8. **Code actions**: organize imports, organize the frontmatter, YAML and TOML, quick fixes.
9. **Refactorings**: extract, inline, rename, leaf and container.
10. **The component reader** and *generate signature* (decision 11).
11. **Formatting**: its rules, then the formatter.
12. **The preview** (decision 13; `editor-support.md`, decision 8).
13. **The release**: binaries, crates.io, `@compmd/lsp` and its platform packages (decision 14).

The editor packages (CMD-26, CMD-27, CMD-29) need tasks 2 to 5 and the grammars.

## Tests

- **Unit**, in each crate: roots and configurations, positions, each conversion to LSP types.
- **The protocol**: a test client drives the binary over stdio (open, change, ask) and checks the answers, for each feature.
- **Fixtures**: `@compmd/vite`'s example app and a folder of documents whose root, packs and registries differ.
- **A registry**: the local registry `@compmd/registry`'s tests run, for task 6.
- **The shared test set** (`compmd-grammars`) for the semantic tokens.

## Not in this change

- Language servers for the languages of rendered blocks: CompMD gives each block's language and range, and an editor package forwards to the writer's server ([content languages](content-languages.md), decision 3).
- Running a package manager or any of a project's scripts.
- A debug adapter, and anything that runs a component's code: the preview renders, the server only reads.
