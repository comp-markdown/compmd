# The developer guide

**Status: proposed.** A guide for developers who put CompMD in an application or write packs for it, as a page of [the website](design.md), `/guide/`, teased on the landing page. The spec says what CompMD *is*; the package READMEs say what each package *has*; the guide says how to *build with them*, start to end.

## Proposed decisions

1. **One guide, one Markdown file in the repository: `developer-guide.md`**, next to the spec, so it reads on GitHub as well as on the site, and changes with the code in the same commits.
2. **It is a page of the site, `/guide/`, made as the spec page is** (the website's decision 6): rendered at build time with the processor's `plainHtml`, code highlighted by Shiki with CompMD's grammar, a table of contents beside the text that marks where the reader is, each heading linked. One renderer for both long pages, with the spec's requirement words marked only in the spec.
3. **Its code runs.** Each code sample is a file in `website/snippets/`, which tests type-check and run, as the landing page's snippets are today. The guide writes the code inline, so it reads anywhere, and a test checks that each block marked with a snippet's name (` ```ts snippet=vite-app `) is that file's text: the guide can't drift from code that works.
4. **The landing page teases it.** "Use it in your app" becomes three tabs of short code, the install line, a live `stars`, and a button to the guide:
   - **In a Vite app**: `vite.config.ts` with `compmd()`, and a document processed and rendered with `virtual:compmd`;
   - **Your own components**: the app's `stars` as a pack folder, its signature, and the line of `compmd.config.json` that trusts it;
   - **In an editor**: `OpenDocument` and splices, as today.
   The header and "Where next" link to the guide.
5. **The website uses `@compmd/vite` itself**, with a `compmd.config.json` that lists its own pack folder (`content/app/`, `stars` in Svelte, trusted, ambient). The live `stars` is that pack, mounted in the page in host mode; the JSX example's frame comes from the site's own `compmd/` instead of the playground's. The site does what the guide teaches.

## The guide's outline

Each part is a few screens, leads with what the reader does, and links to the spec section and the README that go deeper.

1. **What you build with.** The four packages and how a document flows through them: text → `@compmd/processor` (trees, diagnostics, components) → `@compmd/sandbox` and `@compmd/render` (the page) ← `@compmd/vite` (packs, frames, configuration). One figure.
2. **Quick start: a Vite app.** `npm install`, `vite.config.ts`, a document rendered in a page, an input that changes it: about thirty lines, all of them a snippet.
3. **Documents in your app.** `process` for one render, `OpenDocument` for an editor (edits, splices, hints at the caret); diagnostics and their fixes; the files a document reads; `now`, `tz` and inputs; data from URLs (`allowedOrigins`, `sources`) and from the host (`host:`).
4. **Components from packs.** The three trust levels and what each can reach; installing a pack; `compmd.config.json`, key by key; denying packs, limiting their size, offering libraries.
5. **Your app's own components.** A pack folder, a signature, a React or Svelte component that treats its params as untrusted input, an input it sets; permissive components with `host`.
6. **Writing a pack.** `npm create @compmd/pack`, the plain binding, React or Svelte inside `mount`, a code renderer (`--renders`), the test page, publishing with provenance.
7. **The sandbox, closer.** Frame mode and host mode, what each frame may reach, serving `compmd/` (the two headers), editors that render block by block (`mountUse`).
8. **Without Vite.** `@compmd/vite/config` for the configuration, `buildFrames` and `buildLibraries` for the frames, and the Rust crate for a host that isn't a browser.

## Questions, with proposed answers

**G1. Where the file lives.** Proposed: `developer-guide.md` at the repository's root, beside `compmd-spec-v1.md`, and linked from the README's table. The alternative, `docs/guide.md`, puts it among the repository's design documents, which are for the people who work on CompMD, not with it.

**G2. One page or several.** Proposed: one page, as the spec is: one table of contents, one search with the browser's find, one file to read on GitHub. Eight parts of a few screens each fit; if it grows past that, it splits later along its parts.

**G3. Versions.** Proposed: the guide follows `main`, as the landing page and the playground do, and says which package versions it is written for. A versioned guide waits for version 1 of the packages.

**G4. The live `stars` in Svelte or React.** Proposed: Svelte, the site's own framework, so it adds no runtime to the landing page; the guide's own example is React, the more common case, so both bindings are shown.

## Not in this change

- A search box for the guide (the spec's search is its own task, and would serve both).
- Translations.
- API reference generated from the packages' types: the READMEs and TSDoc carry it; the guide links to them.
