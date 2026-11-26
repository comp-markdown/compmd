# CompMD in editors

- **Status:** Approved (2026-11-17). Its parts are built as tasks of their own, in the order below.

CompMD for the editors writers use: VS Code, Neovim, Helix, Zed and Emacs. Each gets highlighting, folding and selection
by structure from a grammar, and the processor's own help (its diagnostics, completion, exact highlighting) from a
language server. The text of other languages a document holds, a renderer's blocks, gets the same help as a file of that
language, as [content languages in editors](content-languages.md) decided. This design covers three things and what
they share: a tree-sitter grammar, a TextMate grammar and a language server; and a preview in the browser that every
editor opens.

## Who uses what

| Editor | Grammar | Language server through | Its package |
|---|---|---|---|
| VS Code | TextMate | the extension's client | `vscode-compmd`, on the Marketplace and Open VSX |
| Neovim | tree-sitter | its built-in client | entries in nvim-treesitter and nvim-lspconfig |
| Helix | tree-sitter | its built-in client | an entry in Helix's `languages.toml`, with its queries |
| Zed | tree-sitter | the extension | `zed-compmd`, in Zed's extensions |
| Emacs (29 and later) | tree-sitter (built in) | Eglot | `compmd-ts-mode`, on MELPA |

Shiki reads TextMate grammars too, so the site, the guides, the playground and the Developer Tools highlight CompMD
(a ```` ```compmd ```` block) with the same grammar VS Code uses: the site's and the guides' examples at build time,
the playground's and the Developer Tools' source in the frames' Shiki, and the books (PDF) as the site's pages are.
A CompMD document's own ```` ```compmd ```` blocks are highlighted the same way, by the sandbox's code highlighting.

## Decisions

1. **Three parts, each doing what it is good at.** A grammar runs in the editor, on every keystroke, without CompMD
   installed: it gives colours, folds and structure from the text alone, as well as a grammar can. The language server
   runs the processor: it knows what the text means (which names are components, which imports resolve, which params a
   signature takes) and gives diagnostics, completion, hover and exact highlighting on top of the grammar's (semantic
   tokens). An editor without the server still has a good grammar, and the server never re-parses CompMD itself.

2. **What the three share, written once:**
   - **The syntax's pieces** (`syntax/`, JavaScript): the regular expressions for a component's name, a param, a
     component line's opener and closer (`:::`, `::`, `::: /name`), an inline component, an interpolation, the
     frontmatter's `md` key and the opt-in syntax (spec §2, §3, §5). The tree-sitter grammar imports them
     (`grammar.js` is JavaScript), and the TextMate grammar is generated from them, so the two read the same text the
     same way.
   - **The token names** (`syntax/tokens.json`): one list of what a span is (a component's name, a param's key, a
     param's value, an expression, an interpolation's braces, a slot's name, a definition's fence), each with its
     tree-sitter capture (`@tag`, `@attribute`), its TextMate scope (`entity.name.tag.compmd`) and its semantic token
     type (`type`, `property`). A component's name is then the same colour in every editor, from a grammar or from the
     server.
   - **The injections** (`syntax/injections.json`): CEL inside an expression and an interpolation; a definition fence's
     language from its info string (`component=chart lang=jsx` is JSX, `compmd` is CompMD again); a renderer's block by
     its fence's language (content languages, decision 7); YAML or TOML for the frontmatter.
   - **The tests**: one test set (`tests/`), drawn from the spec's examples and the conformance set, in which each case
     is a document and the token name each span should have. All three are checked against it: the tree-sitter
     grammar with `tree-sitter query` on its `highlights.scm`, the TextMate grammar by tokenizing with `vscode-textmate`
     (as VS Code does), and the language server by its semantic tokens. A case is written once and checks all three.

3. **The tree-sitter grammar** (`tree-sitter-compmd`): two grammars on [tree-sitter-markdown]'s, as it is split, a
   block grammar and an inline grammar injected into each block's text. Each extends its Markdown grammar
   (`grammar(markdown, { … })`), so CommonMark and GFM stay theirs. The block grammar adds component lines (a container,
   its closer, a slot, a leaf, their params and continuation lines), the frontmatter's `md` key and component
   definitions; the inline grammar adds inline components, interpolation, wiki links (parsed always: a grammar can't
   read the frontmatter's opt-in) and inline math by Pandoc's rule (spec §2.3). Both grammars' external scanners (C)
   are copied from tree-sitter-markdown's, the block grammar's given CompMD's lines and the inline grammar's the tokens
   that need a look ahead (an inline component's `:`, an interpolation's `{{`, math's `$`): a scanner is the one part
   that can't be inherited, and asking upstream for a hook first would hold the grammar up. Expressions are injected as CEL, from a small
   `tree-sitter-cel` beside it (none published covers CEL as spec §8.2 uses it). Its queries: `highlights.scm`,
   `folds.scm` (a container to its closer, a definition fence), `injections.scm` (generated from the injection table),
   and `textobjects.scm` (a component, its params).

4. **The TextMate grammar** (`compmd.tmLanguage.json`): generated from `syntax/` by a script, on VS Code's Markdown
   grammar as its base: CompMD's patterns come first, and `text.html.markdown` takes the rest, as other Markdown
   dialects' grammars do. A TextMate grammar matches line by line, so it sees less than tree-sitter: a container's
   closer is matched by its name, not by nesting, and a param that continues on the next line is matched by the line
   it starts on. Where it can't tell, it colours less, never wrongly; the language server's semantic tokens fill in.

5. **The language server** (`compmd-lsp`): a Rust crate and binary on the processor crate, in this repository, as
   `compmd-host` is, so it is released with the processor and always parses what the spec says. It gives:
   - **diagnostics**: the processor's (spec §10), with their codes, ranges and the spec's section;
   - **semantic tokens**: by the shared token names, from the processor's tree (which names are components, which are
     unknown, which params are a signature's);
   - **completion and hover**: component names from the document's imports and definitions, a signature's params and
     their types, `md.` paths, the built-ins; hover shows a component's signature and description;
   - **go to definition**: an import to its file or pack, a component to its definition;
   - **the content-language blocks**: each block a renderer draws, its language `id` and range (content languages,
     decision 3), as a custom request (`compmd/blocks`), so an editor package forwards requests inside a block to the
     writer's server for that language;
   - **document symbols and folding**: headings, components and definitions, for an outline;
   - **templates**: snippets in completion and code actions that insert a construct ready to fill: a container, a
     multi-column `row`, a leaf or inline component with its signature's params, a slot, a component definition, and a
     signature block for a host-language component (in its fence's language). They are the server's, so every editor
     has the same ones, and an editor package adds none of its own;
   - **formatting**: the document formatted to one canonical form (a component line's params, a continuation's
     indentation, a table's columns), as `textDocument/formatting` and for a range; its rules are written first, as a
     part of the formatter's task, and leave Markdown's own text as the writer wrote it;
   - **refactorings**, as code actions: extract a component's definition into a file of its own (and import it), inline
     a `compmd` component's use as what it expands to, extract a selection into a new component, rename a component
     across the document and the files that import it, and turn a leaf into a container (and back) where its slots
     allow.
   It works on the document's files and installed packs as any processor does (the configuration, `@compmd/vite`'s
   packs); a registry pack's signature comes from `@compmd/registry`'s cache, and a pack it can't load is a diagnostic,
   not a failure. It processes as the user types with the processor's incremental processing
   ([incremental.md](../crates/processor/docs/incremental.md)).

6. **The editor packages are thin.** Each wires the grammar, the language server and the file type (`.comp.md`) into
   one editor, and adds what only that editor needs: `compmd-ts-mode`'s major mode, `zed-compmd`'s manifest, the VS Code
   extension's client and its content-language part (decision 7). Neovim and Helix need no package of their own: an
   entry upstream in nvim-treesitter, nvim-lspconfig and Helix's `languages.toml`, with the queries they keep in their
   own repositories.

7. **Content languages, through all three** ([content-languages.md](content-languages.md)):
   - **in a tree-sitter editor**, a renderer's block is injected by its fence's language, as Markdown's are, so the
     editor's own grammar for that language highlights it (the writer installs it as for any language);
   - **in VS Code**, the TextMate grammar embeds a block's language by its scope (`source.mermaid`), so an installed
     grammar for it works; a pack's own grammar (`compmd.languages`) can't be added to VS Code once it is running (an
     extension declares its grammars up front), so the extension tokenizes the block with the pack's grammar itself
     (`vscode-textmate`, in the extension) and gives the result as semantic tokens, by the scope's nearest token type;
   - **the language servers for them**: the server's `compmd/blocks` gives each block's language, range and the
     language's names elsewhere (`aliases`, `languageId`, `extensions`), and the package forwards completion, hover and
     diagnostics inside it to the server the writer has for that language: VS Code by a virtual document per block (its
     embedded-language pattern), the other editors where they support it, and not at all where they don't.

8. **A preview in the browser, from every editor.** The playground without its editor side: a page that shows the
   document the writer has open, as it renders, with its folder's files and packs, updated as they type.
   - **The language server serves it**, as a host of its own ([hosting CompMD](hosting.md)): it has the document's text as it
     changes, and the processor and `compmd-host` are already its crates, so it serves the page, the bundle, the pack
     store and the frames by `compmd-host`'s routes and headers, as every host does. The page is on one port of
     `127.0.0.1` and the frames on another, so the frames have an origin of their own, as the sandbox needs.
   - **The page is the document's page, full window**, with the host's stylesheet, and the Developer Tools behind a
     tab (or a key): their inspector (`@compmd/devtools`'s inspector, on the view the page renders), with its tabs on
     what the processor and the sandbox make of the document. The text is the writer's editor's, so the page has no
     text of its own; the playground, which has, stays a page of the site (`website/playground/`). It processes the
     text with the processor's WebAssembly as the Developer Tools' editor does, sharing its worker and its splices
     rather than writing them again, so the preview is exactly what the playground shows. It is compiled into the
     server's binary, as `tauri-plugin-compmd` compiles the bundle into an app.
   - **How the text reaches it:** the server sends each change, the document's text and the line being edited, to the
     page as server-sent events; the page renders it and scrolls to the block that holds that line (the renderer marks
     each block with the line it starts on, from the processor's positions). *Settled with the user (CMD-425):* the
     events carry only those (`change`), and `reload` when what the document reads changed; the page fetches the rest
     from the server's routes beside it, as frames fetch `packs/`: `request.json`, the document's request but its text
     (its path, the files and the packs' files the server's processor read, its policy), and `registry/`, the proxy of
     the registries its configuration names (`compmd-host`'s `registry::Proxy`), for a pack the request doesn't hold yet.
     The frames' port serves the project's packs to the frames, by the serving contract.
   - **How it opens:** the server offers it as a command (`compmd.preview`) and a code action (*Open preview*) on any
     line, and opens the system browser at the page; so every editor with a language server client has it, Helix too.
     An editor package adds a button or a key where its editor has one (VS Code's title bar, Emacs's key).
   - **What it trusts:** the page and the frames are the document's host, with the trust its configuration gives
     (`compmd.config.json`: its packs, its registry, its allowed origins), remote images and data asked about as any
     host asks ([remote images](remote-images.md)). Only the writer's machine reaches it: it listens on `127.0.0.1` alone, on a port
     chosen when it starts, at a path with a random token the server opens the browser at, and it refuses a request
     whose `Host` isn't its own (so a web page can't reach it by renaming a domain to `127.0.0.1`).

9. **The language server installs itself, as other language servers do.** A writer installs the editor's package,
   and the server comes with it or is offered:
   - **released** from this repository for macOS, Linux and Windows, on x64 and arm64: binaries on each GitHub
     release with their SHA-256 sums, `compmd-lsp` on crates.io (`cargo install`), and `@compmd/lsp` on npm, with a
     package per platform as an optional dependency (as esbuild and Biome ship theirs), so `npm i -g @compmd/lsp`
     works and an installer that reads npm finds it;
   - **VS Code**: the extension is published once per platform with the binary in it, so it works on install;
   - **Neovim**: an entry in mason.nvim's registry, so `:MasonInstall compmd-lsp` (or mason-lspconfig's
     `ensure_installed`) installs it, after the writer asks for it;
   - **Zed**: the extension downloads the release binary for the platform the first time a document opens, as Zed's
     language extensions do, and checks its sum;
   - **Emacs**: `compmd-ts-mode` asks, when the server isn't found, whether to download it, and downloads the release
     binary for the platform and checks its sum, as lsp-mode does for its servers;
   - **Helix**: has no installer; its entry names `compmd-lsp`, and the guide says how to install it (npm, cargo or
     the release binary).

10. **Where they live:**
   - `comp-markdown/compmd-grammars`: both grammars, `tree-sitter-cel` and what they share (`syntax/`, `tests/`). The
     tree-sitter grammar is in `tree-sitter/`; Helix, Zed and nvim-treesitter each take a grammar from a folder of a
     repository. Published as `tree-sitter-compmd` on npm and crates.io (with the bindings `tree-sitter generate`
     writes), and the TextMate grammar as `@compmd/textmate` on npm, for Shiki and the VS Code extension. Its tests
     take the spec's examples and the conformance set from this repository at a release's tag, and the language
     server's semantic tokens from its release.
   - `comp-markdown/compmd`, this repository: the language server, `compmd-lsp`, released as decision 9 says.
   - `comp-markdown/vscode-compmd`, `comp-markdown/zed-compmd`, `comp-markdown/compmd-ts-mode`: the editor packages,
     each in its own repository, as their stores expect.
   Each grammar names the spec version it parses (a grammar for version 1).

## What it needs

- Dependencies, in `compmd-grammars`: `tree-sitter-cli` (to generate and test), `@tree-sitter-grammars/tree-sitter-markdown`
  (the grammars it extends), the bindings' own (`node-gyp-build`, the `cc` and `tree-sitter-language` crates), and
  `vscode-textmate` with `vscode-oniguruma` for the TextMate tests. In the language server: `lsp-server` and `lsp-types`
  (rust-analyzer's, synchronous, with no async runtime), and for the preview a small HTTP server crate (`tiny_http`,
  synchronous as well). In the VS Code extension: `vscode-languageclient` and
  `vscode-textmate`.
- In this repository: the processor's blocks (content languages, decision 3), the language server and its release
  (binaries and npm packages per platform, in `release.yml`), the preview's page on `@compmd/devtools`, the renderer's line on
  each block, and the CompMD grammar in the frames' Shiki and the site's build.
- In packs: `compmd.languages`, checked (content languages, decision 2).
- CI in `compmd-grammars`: generate, check the generated parser is committed (tree-sitter grammars commit
  `src/parser.c`), and run the test set on both grammars and the released server.

## The order

The shared `syntax/` and `tests/` first, then the tree-sitter grammar and the TextMate grammar (either order), the
processor's blocks and the language server, then the preview, then the editor packages, each needing its grammar, the
server and the preview. CompMD's highlighting with Shiki follows the TextMate grammar; the server's release, its
templates, the formatter and the refactorings follow the server, each a task of its own.

[tree-sitter-markdown]: https://github.com/tree-sitter-grammars/tree-sitter-markdown
