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

4. **The editor sees what the site will render: diagnostics come from the document's own configuration, nothing else.** The server resolves imports and loads packs exactly as a host with that configuration does. An import the site couldn't load is F026 in the editor too, never quietly fixed by what the editor can reach. What the editor can reach beyond that (decision 7) only *suggests*: completions, and code actions that add an import or a registry. It never changes a diagnostic.

   **One exception: CompMD's own packs work with no configuration** (agreed in the design's review). The editor resolves `@comp-markdown` from npm by default, so an import of one of its packs loads, completes and hovers at once. When the document's own configuration couldn't load that pack (it isn't installed, and no registry names the scope), the import gets a **warning**, not F026: *your site can't load `@comp-markdown/mermaid`: install it, or add a registry*, with a quick fix that adds the registry to `compmd.config.json` (Q4). So the editor never hides a step the site needs: it names it and offers it.

5. **Packs come from five places, in the host's order** ([hosting](hosting.md), decision 4):

   | Place | Example | How the server gets it | Diagnostics |
   |---|---|---|---|
   | **built in** | `@compmd/core` | the processor has it | always |
   | **the document's own files** | `./chart.jsx`, `./cards.comp.md`, a `component=` fence | the files under the root, read as the editor holds them (an unsaved file's text included) | always |
   | **folder packs** | `./components` in `compmd.config.json` | read from the folder, relative to the configuration's folder | as the configuration lists them |
   | **installed packs** | `@comp-markdown/mermaid` in `node_modules` | found as `@compmd/vite` finds them: the root's dependencies that are packs, their integrity from the lock file | always available, strict unless listed |
   | **registry packs** | any pack an import names, from npm or a private registry | fetched by the configuration's `registries`, checked and cached (decision 6); `@comp-markdown` from npm by default | when the configuration names a registry for its scope; a `@comp-markdown` pack it can't load is a warning with its fix (decision 4) |

   **Every source is one abstraction, the processor's own.** The processor already takes packs through one trait, `compmd::host::Packs` (a pack's releases, a pack file by its place, and `pending` for one on its way), and a document's files through `Files`. Each source implements `Packs`, and one chain puts them in the configuration's order:

   | Implementation | Source | Crate |
   |---|---|---|
   | the processor's own | built in (`@compmd/core`) | `compmd` |
   | `FolderPacks` | folder packs, by the configuration | `compmd-project` |
   | `InstalledPacks` | installed packs, by `node_modules` and the lock file | `compmd-project` |
   | `FetchedPacks` | registry packs: the releases resolved for a document from the cache or its registries (`compmd-host`'s `registry::resolve`, H1), `pending` while a fetch is on its way | `compmd-lsp` |
   | `HostPacks` (exists) | a built bundle and its store, for a host that serves one | `compmd-host` |
   | `ChainedPacks` | the sources above, the first that has a pack giving it | `compmd-host` |

   The document's own files are the processor's `Files`: the editor's implementation gives an open document's text as the editor holds it, and the disk's otherwise. A host composes what it has: the server chains the folder, installed and registry packs; `tauri-plugin-compmd` its bundle and store; a command line the same as the server. A new source (a host's marketplace, a workspace's packs) is one more implementation, and nothing that reads packs changes.

   **Known packs are one more method, not another abstraction.** What completion offers beyond the document (decision 7) is a pack source's listing: `Packs` gains `fn known(&self) -> Vec<KnownPack>` (each release a source holds, with its description), empty by default, so a host that doesn't suggest (a site) implements nothing. The installed and folder sources list what they hold; the registry source lists its suggested scopes; a chain lists each source's but what an earlier one hides. The server reads each known release's exported components and their signatures by processing a document that imports them all, as it reads any pack (CMD-430: the processor takes `'static` packs, so a source can't process itself).

   `compmd-project` reads the project: the configuration, the folder and installed sources, and the lock file's integrity. It sits beside `compmd-host`, so the server, a command line, `tauri-plugin-compmd` and `@compmd/vite` (as WebAssembly, Q3) all read a project with one implementation.

6. **Registry packs through a registry client in Rust, as `hosting.md` already plans (H1, option c).** The resolver's loop, ranges matched as the processor matches them, the policy before any download, the integrity checks and the tar reader move into `compmd-host`, which `@compmd/registry` then uses as WebAssembly. The server is that client's first user. It takes an HTTP client (a small blocking one, `ureq`) and a cache: a folder in the user's cache directory (`~/.cache/compmd/packs` and its platform equivalents), keyed by a release's pack, version and integrity, as `PackCache` is. The server fetches the way the configuration says (`fetch: "host"` or `"direct"`): either way, from the writer's own machine.

7. **What the editor offers beyond the document's configuration: the known packs, set by the writer.** Completion and auto-import can offer a pack the document doesn't import yet, for a component's name or a code block's language (the code renderer a known pack exports that renders it, spec §7.7), from a list of **known packs**:
   - the installed and folder packs (no network);
   - the packs of the **suggested scopes**: `@comp-markdown` by default, CompMD's own packs, with others the writer adds. The server lists a scope's packs from its registry's search (npm's `/-/v1/search`, by the scope and the keyword `compmd-pack`, which only packs use, [packs from a registry](packs-from-a-registry.md) R6; a registry without search offers only what is installed or cached), fetches each listed pack's latest release, and reads the names and signatures its document exports. A scope is searched once a session, however many documents suggest it; a scope the configuration names no registry for is searched on npm;
   - a writer turns this off (`compmd.packs.network: false`), and the server then suggests only what is on the machine.

   The writer adds scopes in the editor's settings (decision 12), and a project adds its own for everyone who opens it, in `compmd.config.json`'s `editor.suggest` (Q2): what is offered is CompMD's, then the project's, then the writer's. Choosing a known pack from a registry doesn't make the site able to load it: auto-import adds the import and, when the site couldn't load the pack, asks to add its registry to `compmd.config.json` (Q4). For a pack outside `@comp-markdown`, the import's F026 says what the configuration needs, with the same fixes (Q4).

8. **One processor per open document, edited as the writer types.** A document's worker keeps its expanded processor, applies each change as a `TextEdit` (the processor's incremental processing, [incremental.md](../crates/processor/docs/incremental.md)), and publishes the diagnostics after each. A change to a file the document imports, to the configuration or to `node_modules` reloads the documents that use it (`workspace/didChangeWatchedFiles`, which the server registers for). Hints, completions and hovers are lookups in the processor's state ([positional hints](../crates/processor/docs/positional-hints.md)), never another run.

9. **Every feature is the processor's, given in LSP terms.** The server writes no CompMD logic of its own: what a name stands for, what a param takes, what a function does and what a block's language is come from the processor (`hint_at`, `components`, `rendered_blocks`, `functions`, the trees). Where a feature needs something the processor doesn't give yet, the processor gains it, for the WebAssembly package and the Developer Tools too. The features and their processor APIs are in [the features](#the-features).

10. **Templates, code actions and refactorings are the server's, so every editor has the same ones** (`editor-support.md`, decision 5). They come from the signatures the document can name, with params filled from their defaults and the values their types take first. The ideas in IN-1 are each placed in [what IN-1 asked for](#what-in-1-asked-for).

11. **A host-language component's signature is written from its code by a component reader in Rust.** Generating or updating a `signature` block (IN-1) reads a component's props: `.jsx`, `.tsx`, `.js` and `.ts` with `oxc` (a Rust parser for JavaScript and TypeScript), a `.svelte` file's `<script>` with the same parser (Svelte 5's `let { … }: Props = $props()`). It reads what the source states, as `@compmd/vite`'s analysis does (W104): a prop's name, a TypeScript type it can map, its default. A prop it can't read is left for the writer, with a comment, never guessed. The reader is a crate of its own, `compmd-components`, which also gives the processor W104 for a document's own components and replaces `@compmd/vite`'s analysis (Q5).

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
- **Auto-import** (IN-1): a component name the document can't name yet, offered from the known packs (decision 7), with `additionalTextEdits` that add the import to the frontmatter's `md.import`, in the frontmatter's own syntax, YAML or TOML. The same for a code block's language: after ` ``` `, the languages the document's code renderers render, then those the known packs' renderers render, each importing its renderer (the processor's `Subject::Language`, on every fence's language, an empty one too). Two packs rendering one language are both offered.
- `textDocument/hover`: what `hint_at`'s subject says, as Markdown: a component's origin and signature, a param's type, default and doc, a path step's type and what it is, a function's signatures, description and first example, a frontmatter key's meaning, a type's values, and the diagnostics at the place.
- `textDocument/signatureHelp`: inside an expression's call, the function's signatures with the current argument; inside a component's param list, its signature's params.
- `textDocument/definition`: an import's `from` to its file or pack document; a component's name to its definition fence, or to the file or pack that exports it; an interpolation's `md.data.<name>` to its data import.

### Blocks, symbols, folding

- `compmd/blocks` (a custom request): `rendered_blocks`, each block's language `id`, its range and the language's other names (content languages, decision 3), for an editor package to forward requests inside it to the writer's server for that language.
- `textDocument/documentSymbol`: headings, containers, leaves and definitions, nested as the document nests them.
- `textDocument/foldingRange`: the frontmatter, containers (an opener to its closer), definition fences, rendered blocks.
- `compmd/drawing` (a custom request) and `compmd/drawing/refresh`: the plan an editor draws a document in place with, in [a document drawn in the editor](editor-rendering.md).

### Templates, code actions, refactorings (IN-1)

- **Templates** are completions with snippets (`insertTextFormat: Snippet`), at the start of a line: a container with its signature's required params as tab stops and the rest in a choice, a `row` of columns, a leaf, an inline component, a slot of the container the line is in, a component definition (`compmd`, or a host language's fence with its `signature` block), an `each` and an `if`. From the signatures the document can name, never a fixed list.
- **Code actions** (`textDocument/codeAction`), each of a kind an editor groups:
  - `quickfix`: a diagnostic's `fix`; import a known pack for an unknown name; *Import `<pack>` to render this block*, anywhere in a code block whose language no renderer the document names renders but a known pack's does (no diagnostic: such a block is valid and shows as code); add the registry of a pack the site can't load to `compmd.config.json` (decision 4, Q4).
  - `source.organizeImports`: `md.import` sorted by source, names merged and sorted, unused names dropped, in the frontmatter's syntax.
  - `source.organizeFrontmatter.compmd`: the frontmatter's keys in a canonical order (the author's keys first, then `md`'s in the spec's order).
  - `source.addFrontmatterKey.compmd`: each common key a document lacks (Q7), offered on its own, empty (`""`, `[]`; `md.lang` its default, `en`) and inserted where it goes. A kind apart from organizing, so that organizing on save adds nothing.
  - `refactor.rewrite`: the frontmatter from YAML to TOML and back (spec §2.1 reads both), keeping comments where the other syntax has them.
  - Organizing and changing the syntax write the whole frontmatter again, laid out one way for each syntax: a value's quotes and its flow or block style can change with them. A comment on a line of its own stays above the key after it.
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
  project/          compmd-project: the root, compmd.config.json, folder and installed packs, lock files' integrity (native and WebAssembly)
  components/       compmd-components: a host-language component's props, read from its source with oxc (W104, signatures)
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

**Q1. The crate for reading a project.** *Settled in the review:* `compmd-project`, a crate of its own. `compmd-host` reads a built bundle and serves it; reading a project's sources (its configuration, `node_modules`, a lock file) is what a build and an editor do, not a host.

**Q2. A project's own suggested scopes.** *Settled in the review: yes, as proposed below.* Should a project be able to name, for everyone who opens it, the scopes whose packs completion and auto-import offer?

- *The problem.* The suggested scopes (decision 7) are the writer's own setting: `@comp-markdown` by default, and whatever else they add in their editor. That suits a writer's taste, but not a team's. Take a publication whose site draws charts with its own packs, published on npm or a private registry as `@acme/charts`, `@acme/maps`. A new writer clones the repository and opens a post: completion offers CompMD's packs, but not `@acme`'s, until someone tells them to add `@acme` to their editor's settings, in whichever editor they use (each has its own settings file and syntax).
- *Proposed:* the project says it once, in the file every host already reads: `compmd.config.json` takes an `editor` key, `{ "editor": { "suggest": ["@acme"] } }`. Everyone who opens the project, in any editor, is offered `@acme`'s packs, with nothing to set up.
- *How the lists combine.* What is offered is the union, in this order: CompMD's own (`@comp-markdown`), then the project's (`editor.suggest`), then the writer's (their setting). A writer can't remove a project's scope, only turn the network off altogether (`compmd.packs.network: false`), which then offers only what is installed or cached.
- *What it doesn't change.* Only suggestions: diagnostics still come from the configuration's packs and `registries` (decision 4). Listing `@acme` under `editor.suggest` doesn't make the site load `@acme`'s packs. Auto-importing one asks to add its registry to `compmd.config.json`, and until then its import says which registry the configuration needs (Q4).
- *Where its packs come from.* A suggested scope is listed from the registry the configuration names for it (`registries["@acme"]`), else from npm. A private registry's token is never in the configuration (it is the writer's own `.npmrc`, read as npm reads it); without one, that scope offers only what is installed.
- *What it costs.* The configuration's schema gains one key, which every host but the editor ignores (`@compmd/vite`, `buildBundle`, the Tauri plugin), and `readConfig` checks it like the others: a list of scopes.
- *Alternatives.* A file of the editor's own in the project (`.compmd/editor.json`): one more file to know about, beside a configuration that already names the project's packs and registries. Each editor's project settings (`.vscode/settings.json`, `.nvim.lua`, `.helix/languages.toml`): written once per editor, and only some editors read them.

**Q3. `@compmd/vite` on `compmd-project`.** *Settled in the review: now.* The Vite plugin reads projects through the Rust crate, as WebAssembly, as `@compmd/registry` will its rules (H1), so a project is read by one implementation:
- `compmd-project` reads files through a trait of its own (a file's text, a folder's entries), which the native build implements with `std::fs` and the WebAssembly build with functions `@compmd/vite` passes in (Node's `fs`). The crate brings no file system of its own, as the processor brings none.
- `@compmd/vite` keeps its API (`loadApp`, `readConfig`, `policyOf`, `buildBundle`); their bodies call the crate. Its own TypeScript for the configuration, finding packs and the lock files goes. *Settled with the user (CMD-414):* the crate's WebAssembly is in `@compmd/host`'s module (`compmd-host-wasm`), which the plugin already depends on and no reader's page loads. Reading a found pack's files for the bundle stays `@compmd/vite/pack-folder`'s, which `@compmd/pack` shares. `@compmd/vite/config` then needs that module everywhere: in Node, `@compmd/host/project` loads it the first time it is used; a page or a worker loads it first (`await init()` from `@compmd/host`), a breaking change for that use, said in the release notes.
- Its fixtures become the crate's, run natively and through the plugin.
- It is the task after `compmd-project` (see [tasks](#tasks)).

**Q4. A pack the site can't load yet.** *Settled in the review: a runtime dependency, with the registry added on the writer's word, and nothing installed.* When auto-import picks a pack the document's configuration couldn't load:
1. **A runtime dependency, by default.** The import is added, and the editor resolves the pack (decision 4). The server asks the writer at once (`window/showMessageRequest`): *Your site loads packs from npm at runtime only if `compmd.config.json` names it. Add npm for `@comp-markdown`?*, with **Add** and **Not now**.
2. **Add:** the server edits `compmd.config.json` with `workspace/applyEdit`, so the edit shows and can be undone. It adds `registries["@comp-markdown"]`, and every host with that configuration then fetches the pack at runtime, checked against its integrity. The registry's `fetch` is `"host"` when the project has a server to proxy it (it uses `@compmd/vite`, or it already proxies a registry), else `"direct"`. The question says which, and for `"direct"` that npm then sees readers' addresses (spec §15), so it is the writer's stated choice. `fetch` is the configuration's, for all its registries: one it already states, or that its registries already imply (`"host"`, the default), stays, and `"fetch": "direct"` is written only for a configuration that had neither. The edit is made where the configuration is (`compmd.config.json`, or `package.json`'s `compmd`), or creates `compmd.config.json`, and leaves the rest of the file as written. The server asks once per project in a session, and only a client that applies its edits.
3. **Not now:** the import keeps the warning, with adding the registry as its quick fix.

The server never installs a pack nor runs a package manager, which would run a project's scripts; installing stays the writer's step.

**Q5. W104 for a document's own components.** *Settled in the review: now.* The component reader (decision 11) gives the processor W104 for a document's own `./Chart.jsx`, not only for listed packs:
- the processor gains an optional input, a component's props as the reader found them, which it checks against the component's signature (W104), as `@compmd/vite` does today for listed packs;
- the reader is a crate of its own (`compmd-components`, on `oxc`), which the server uses natively and the WebAssembly package exposes, so the Developer Tools and the Playground report W104 for a document's own components too. *Settled with the user (CMD-415):* its WebAssembly is a module of its own, `@compmd/processor/components`, loaded only by what checks components (the Developer Tools, the Playground, `@compmd/vite`): the processor's module, which every reader's page loads, stays within its budget (808 KB of 1 MB gzipped), and the processor's new input is in it, so W104 works wherever the props are given;
- `@compmd/vite`'s analysis (`analysis.ts`, on TypeScript's compiler and Svelte's) moves to the same reader, so W104 has one implementation. What the reader reads is what the source states, as today; a prop whose type it can't map is left unchecked, never guessed.
- It is a task of its own, before the code actions that generate a signature, which use the same reader.

**Q6. An unknown component's token.** How semantic tokens colour a name that stands for no component (F002), such as a typo, `:: catd[…]`. *Proposed:* `component-name` with a modifier `unknown`, which themes can colour (dimmed, or with a wavy underline), beside the diagnostic.

**Q8. The name a new component gets.** The refactorings that make a component (extract a selection, extract an inline component into a file) need a name before the writer chooses one. *Proposed:* `myComponent` (`myComponent2` when it is taken), and the server then starts a rename on it at once (it returns the edit with the command `editor.action.rename` where the client has it), so the writer types the real name straight away, and every use follows. A file extracted takes the name too: `myComponent.comp.md`, `MyComponent.jsx`.

**Q7. "Common" frontmatter keys.** *Settled in the review, as proposed:* the keys the spec's examples and the Author's Guide use for posts and notes (`title`, `description`, `date`, `tags`, `md.lang`), offered, never added unasked.

## Tasks

The work, in order; each is a task with its own review:

1. **`compmd-project`**: the root, the configuration (with `editor.suggest`, Q2), the `Packs` sources for folder and installed packs, lock files' integrity, natively and as WebAssembly; checked against `@compmd/vite`'s fixtures.
2. **`@compmd/vite` on `compmd-project`** (Q3): the plugin reads projects through the crate's WebAssembly; its own TypeScript for them goes.
3. **The component reader, `compmd-components`** (decision 11, Q5): props from source with `oxc`; the processor's W104 for a document's own components; `@compmd/vite`'s analysis moved to it.
4. **The server's core**: the binary, documents and their workers, incremental processing, diagnostics, settings, watched files.
5. **Semantic tokens**, checked by `compmd-grammars`' test set.
6. **Completion, hover, signature help, go to definition**, with auto-import from installed packs.
7. **`compmd/blocks`, document symbols, folding.**
8. **The registry client in Rust** (H1), then the known packs from registries and `@comp-markdown` with no configuration (decisions 4 and 7).
9. **Templates.**
10. **Code actions**: organize imports, organize the frontmatter, YAML and TOML, quick fixes, *generate signature* (on task 3's reader).
11. **Refactorings**: extract (Q8's default name), inline, rename, leaf and container.
12. **Formatting**: its rules, then the formatter.
13. **The preview** (decision 13; `editor-support.md`, decision 8).
14. **The release**: binaries, crates.io, `@compmd/lsp` and its platform packages (decision 14).

The editor packages (CMD-26, CMD-27, CMD-29) need tasks 4 to 7 and the grammars.

## Tests

- **Unit**, in each crate: roots and configurations, positions, each conversion to LSP types.
- **The protocol**: a test client drives the binary over stdio (open, change, ask) and checks the answers, for each feature.
- **Fixtures**: `@compmd/vite`'s example app and a folder of documents whose root, packs and registries differ.
- **A registry**: the local registry `@compmd/registry`'s tests run, for task 8.
- **The shared test set** (`compmd-grammars`) for the semantic tokens.

## Not in this change

- Language servers for the languages of rendered blocks: CompMD gives each block's language and range, and an editor package forwards to the writer's server ([content languages](content-languages.md), decision 3).
- Running a package manager or any of a project's scripts.
- A debug adapter, and anything that runs a component's code: the preview renders, the server only reads.
