# Design documents

The designs behind the spec and its implementation. Each starts with its status and leads with its decisions. A topic design decides *what* CompMD does and *why*; a build design decides *how* one package does a part of a topic design, and names the part it builds.

## Topics

- [Components from apps and packs](components-from-apps-and-packs.md): where component code comes from (an app, a pack, the registry), and at what trust; the parent of most designs below.
- [Packs from a registry](packs-from-a-registry.md): how a host lets documents use packs straight from an npm registry, what a registry pack is, and how packs share libraries.
- [Any npm package, as published](npm-packages.md): proposed, a package's ES module tree resolved by the registry and bundled once in the compile frame, so a pack imports `d3` by its own name.
- [What component code may import](what-component-code-may-import.md): what a bare import in a component resolves to: a runtime, a library the host offers, or a library pack.
- [Code renderers](code-renderers.md): a component that also draws the code blocks of one language (spec §7.7).
- [Handling pack loading states](handling-pack-loading-states.md): a placeholder, for telling a pack on its way from one that won't load.
- [Content languages in editors](content-languages.md): a pack's grammars for the languages its code renderers read, and each block's language and range, so editors highlight them and choose a language server for them.
- [CompMD in editors](editor-support.md): a tree-sitter grammar, a TextMate grammar and a language server, with what they share (the syntax's pieces, the token names, the injections, one test set), and a preview in the browser, for VS Code, Neovim, Helix, Zed and Emacs.
- [Remote images, asked about](remote-images.md): how the renderer and the sandbox ask the reader before a document's remote images, in both modes, with the data's answers.
- [Reading CompMD with pandoc](pandoc.md): proposed, the processor's expanded tree written as pandoc's JSON, each component as it prints, by a `compmd-pandoc` command and a pandoc reader in a repository of their own, so pandoc turns documents into PDF, Word and EPUB.
- [A blog in CompMD](blog-tool.md): `@compmd/blog`, the command `cblo`, a folder of CompMD documents made a static blog (posts, tags, a feed, comments), which the website's blog uses.
- [Wireframes in CompMD](wireframe-pack.md): the ways a pack could draw wireframes (CompMD components, a small language of its own, Salt, Excalidraw, a service), and the one chosen: Wireloom's syntax as a grammar of our own, drawn as HTML and CSS by a code renderer pack.
  - [The wireframe language](wireframe-language.md): its grammar in ABNF, the parts drawn first, how each is drawn, its errors and examples, and where it differs from Wireloom.
- [Printing a document](printing.md): proposed, what a document looks like on paper: its components, its colours, its page breaks, and what a processor without a browser prints.
- [Hosting CompMD](hosting.md): what every host does (the configuration, the bundle, the pack store, serving, the page's configuration, the check), which of it is shared code, and the adapter a platform writes.

## How the packages build them

- [Running components in the sandbox](running-components-in-the-sandbox.md): how `@compmd/sandbox` runs code at the three trust levels; builds the levels of *components from apps and packs*.
  - [The sandbox's frame protocol](sandbox-protocol.md): the messages between its windows.
  - [Libraries in the sandbox](libraries-in-the-sandbox.md): where a library the sandbox loads (Shiki, Temml) is built, and how a host-mode page loads the frames' copy, so an app ships it once; with the steps for adding one.
- [The Tauri plugin](tauri-plugin.md): `tauri-plugin-compmd`, the Tauri adapter of *hosting CompMD*: the bundle compiled into the app and served from a scheme of its own, and the example app that checks it in a real webview.
- [The configuration and `@compmd/vite`](vite.md): `compmd.config.json` and the Vite plugin, for packs an app installs; builds H3 of *components from apps and packs*.
- [`@compmd/create-pack`, the pack template](create-pack.md): `npm create @compmd/pack`; builds S2 of *components from apps and packs*.
- [Resolving registry packs](resolving-registry-packs.md): how `@compmd/registry` resolves, fetches and checks packs; builds decisions 1 to 3 and 5 of *packs from a registry*.
- [Sharing library packs](sharing-library-packs.md): how the processor, `@compmd/registry` and `@compmd/sandbox` let packs share a library pack; builds decision 8 of *packs from a registry*.
- [Writing a pack](writing-a-pack.md): [`@compmd/pack`](../pack/README.md), which holds what every pack would otherwise copy (the build, the publish check, the test page, component helpers and a testing kit), and the host contract's `scheme` and `fail`.
- [The Developer Tools](devtools.md): `@compmd/devtools`, the playground's tools as a part any page embeds, with the sandbox's observer and tabs for its frames, a console and the messages.
- [The Mermaid pack](mermaid-pack.md): `@comp-markdown/mermaid`, a code renderer as a registry pack; builds the Mermaid example of *code renderers*.

The processor's own designs are next to it, in [`processor/docs/`](../processor/docs/): [incremental processing](../processor/docs/incremental.md), [param values](../processor/docs/param-values.md) and [positional hints](../processor/docs/positional-hints.md).

## The repository

- [CI on GitHub](ci.md): the checks every push and pull request runs.
- [Releases](releases.md): what is released, how it is versioned, and what a release does.
