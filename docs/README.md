# Design documents

The designs behind the spec and its implementation. Each starts with its status and leads with its decisions. A topic design decides *what* CompMD does and *why*; a build design decides *how* one package does a part of a topic design, and names the part it builds.

## Topics

- [Components from apps and packs](components-from-apps-and-packs.md): where component code comes from (an app, a pack, the registry), and at what trust; the parent of most designs below.
- [Packs from a registry](packs-from-a-registry.md): how a host lets documents use packs straight from an npm registry, what a registry pack is, and how packs share libraries.
- [Any npm package, as published](npm-packages.md): proposed, a package's ES module tree resolved by the registry and bundled once in the compile frame, so a pack imports `d3` by its own name.
- [What component code may import](what-component-code-may-import.md): what a bare import in a component resolves to: a runtime, a library the host offers, or a library pack.
- [Code renderers](code-renderers.md): a component that also draws the code blocks of one language (spec §7.7).
- [Code and what it draws](code-and-output.md): proposed, `render=source|output|both` on a rendered code block, so a post shows a block's code beside its drawing; and, later, `compmd` blocks rendered the same way.
- [Handling pack loading states](handling-pack-loading-states.md): a placeholder, for telling a pack on its way from one that won't load.
- [Content languages in editors](content-languages.md): a pack's grammars for the languages its code renderers read, and each block's language and range, so editors highlight them and choose a language server for them.
- [CompMD in editors](editor-support.md): a tree-sitter grammar, a TextMate grammar and a language server, with what they share (the syntax's pieces, the token names, the injections, one test set), and a preview in the browser, for VS Code, Neovim, Helix, Zed and Emacs.
  - [The language server](language-server.md): proposed, `compmd-lsp` in Rust: a document's root and packs (built in, its folder, installed, registries), the known packs editors suggest, each feature on the processor's APIs, and the inbox's ideas for it.
  - [A document drawn in the editor](editor-rendering.md): proposed, headings, code blocks, tables, callouts and containers drawn in place, as render-markdown.nvim draws Markdown: the language server's plan of six primitives, each editor drawing what it can (Neovim, Emacs, VS Code), and plain LSP for the rest (Helix, Zed); plain Markdown files too, with links and headings, every choice a setting, and what Markdown and CompMD writers get in each editor.
- [Remote images, asked about](remote-images.md): how the renderer and the sandbox ask the reader before a document's remote images, in both modes, with the data's answers.
- [Reading CompMD with pandoc](pandoc.md): proposed, the processor's expanded tree written as pandoc's JSON, each component as it prints, by a `compmd-pandoc` command and a pandoc reader in a repository of their own, so pandoc turns documents into PDF, Word and EPUB.
- [A blog in CompMD](blog-tool.md): `@compmd/blog`, the command `cblo`, a folder of CompMD documents made a static blog (posts, tags, a feed, comments), which the website's blog uses.
- [Wireframes in CompMD](wireframe-pack.md): the ways a pack could draw wireframes (CompMD components, a small language of its own, Salt, Excalidraw, a service), and the one chosen: Wireloom's syntax as a grammar of our own, drawn as HTML and CSS by a code renderer pack.
  - [The wireframe language](../compmd-packs/wireframe/LANGUAGE.md), in the pack: its grammar in ABNF, the parts drawn first, how each is drawn, its errors and examples, and where it differs from Wireloom.
- [The first components](first-components.md): proposed, which components CompMD should have first, from what documentation frameworks ship, the component packages used most and a page's widgets: the basics built in (tabs, a collapsible, steps), cards and figures without code, charts, a calendar, the weather, music and guitar, jq, and a permissive pack of embeds; with each one's package and how it reaches what is outside the document.
  - [What `compmd` definitions need for the no-code components](no-code-components.md): proposed, a param's destination resolved against the caller, `md.fallback`, `md.slots`, an inline invocation's box, `stretch-link`, `shadow` and `rotate`.
  - [Gradients in `background`](background-gradients.md): accepted, a linear or radial gradient as a box's background, checked part by part, with directions and angles that follow the text's, for the sticky note's cut corner.
- [Printing a document](printing.md): proposed, what a document looks like on paper: its components, its colours, its page breaks, and what a processor without a browser prints.
- [What a component may assume about its box](component-environment.md): approved, the host contract promising that inline styles apply in a box's document, kept by the frames' policy or the host's page, so a library that styles and measures as it draws works without workarounds; with what was tried and rejected.
- [Hosting CompMD](hosting.md): what every host does (the configuration, the bundle, the pack store, serving, the page's configuration, the check), which of it is shared code, and the adapter a platform writes.

## How the packages build them

- [Running components in the sandbox](running-components-in-the-sandbox.md): how `@compmd/sandbox` runs code at the three trust levels; builds the levels of *components from apps and packs*.
  - [The sandbox's frame protocol](sandbox-protocol.md): the messages between its windows.
  - [Libraries in the sandbox](libraries-in-the-sandbox.md): where a library the sandbox loads (Shiki, Temml) is built, and how a host-mode page loads the frames' copy, so an app ships it once; with the steps for adding one.
  - [The host's code highlighter](host-highlighter.md): proposed, the sandbox taking its highlighter from its host in both page modes, frames asking the host's page for each block's tokens, and whether the default Shiki moves there too, measured on pages of 1, 10 and 40 frames.
  - [WebAssembly in the frames](webassembly-in-the-frames.md): the frame documents allowing WebAssembly, so the frames highlight code with Oniguruma: what it gives component code against each threat, and what else was weighed.
- [The Tauri plugin](tauri-plugin.md): `tauri-plugin-compmd`, the Tauri adapter of *hosting CompMD*: the bundle compiled into the app and served from a scheme of its own, and the example app that checks it in a real webview.
- [The configuration and `@compmd/vite`](vite.md): `compmd.config.json` and the Vite plugin, for packs an app installs; builds H3 of *components from apps and packs*.
- [Pack settings by page in `@compmd/vite`](vite-per-page-config.md): each page of a Vite project gets the nearest `compmd.config.json`, so pages of readers' documents and pages of the app's own run with different trust.
- [`@compmd/create-pack`, the pack template](create-pack.md): `npm create @compmd/pack`; builds S2 of *components from apps and packs*.
- [Resolving registry packs](resolving-registry-packs.md): how `@compmd/registry` resolves, fetches and checks packs; builds decisions 1 to 3 and 5 of *packs from a registry*.
- [The registry rules in `compmd-host`](registry-rules.md): those rules once, in Rust, as a state machine every host drives with its own I/O; hosting's H1, option (c).
- [Sharing library packs](sharing-library-packs.md): how the processor, `@compmd/registry` and `@compmd/sandbox` let packs share a library pack; builds decision 8 of *packs from a registry*.
- [Writing a pack](writing-a-pack.md): [`@compmd/pack`](../npm-packages/pack/README.md), which holds what every pack would otherwise copy (the build, the publish check, the test page, component helpers and a testing kit), and the host contract's `scheme` and `fail`.
  - [Sizing a component in its box](sizing-components.md): proposed, a component fitting its box, which the use's common params size, a pack's box defaults on its signature fence, and the sandbox's frames giving a use exactly its box.
  - [An image cropped to its box](cropping-images.md): approved, a box holding one image fitting it by `object-fit` and `object-position`, so the box's height crops it; W125 on any other box.
  - [Type in a box, and fonts of its own](type-params.md): approved, ten type params with tokens (font size, weight, style and family, colour, alignment, line height, letter spacing, transform, decoration), inherited, a component's box passing them on, fonts from a pack or the document's folder, W126 for two colours that don't contrast.
- [Wrapping an npm package as a pack](wrapping-an-npm-package.md): proposed, `npm create @compmd/pack -- --from <package>`: the glue written once, the signature generated at each build from the package's types, and what a strict frame refuses; with [its spike](../spikes/wrap-npm-package/README.md), four packages wrapped and run.
- [The Developer Tools](devtools.md): `@compmd/devtools`, the playground's tools as a part any page embeds, with the sandbox's observer and tabs for its frames, a console and the messages.
- [The Mermaid pack](mermaid-pack.md): `@comp-markdown/mermaid`, a code renderer as a registry pack; builds the Mermaid example of *code renderers*.

The processor's own designs are next to it, in [`crates/processor/docs/`](../crates/processor/docs/): [incremental processing](../crates/processor/docs/incremental.md), [param values](../crates/processor/docs/param-values.md) and [positional hints](../crates/processor/docs/positional-hints.md).

## The repository

- [CI on GitHub](ci.md): the checks every push and pull request runs.
- [Releases](releases.md): what is released, how it is versioned, and what a release does.
- How code is written here: [Rust](code/rust.md), [Svelte and the TypeScript packages](code/svelte.md), [Playwright](code/playwright.md); the overall rules are in [AGENTS.md](../AGENTS.md).
