# The CompMD Developer Guide

**Status: proposed.** A book for developers who put CompMD in an application or write packs for it: *the CompMD Developer Guide*. Its chapters are in the repository, its pages on [the website](design.md) at `/guide/`, and the landing page teases it. The spec says what CompMD *is*; the package READMEs say what each package *has*; the guide teaches how to *build with them*, from a first page to a published pack.

## Proposed decisions

1. **The guide is a book in a folder of its own, `guide/`**, beside the spec: a contents page (`guide/README.md`) and one Markdown file per chapter, grouped in parts. It reads on GitHub as a book does, changes with the code in the same commits, and grows a chapter at a time.
2. **`guide/` is a workspace package**, so its code is real: each code sample is a file in `guide/snippets/`, type-checked and run by the package's tests against the repository's packages. A chapter writes its code inline, so it reads anywhere, and a test checks that each block marked with a snippet's name (` ```ts snippet=quick-start/main `) is that file's text: the guide can't drift from code that works.
3. **On the site, the guide is pages of its own, `/guide/` and `/guide/<chapter>/`**, made at build time with the spec page's renderer (the website's decision 6): the processor's `plainHtml`, code highlighted by Shiki with CompMD's grammar, headings linked. Each page has the book's chapters beside the text, its own sections under the current chapter, and links to the chapters before and after it. `/guide/` is the contents page.
4. **The landing page teases it.** "Use it in your app" becomes three tabs of short code from the guide's snippets, the install line, a live `stars`, and a button to the guide:
   - **In a Vite app**: `vite.config.ts` with `compmd()`, and a document processed and rendered with `virtual:compmd`;
   - **Your own components**: the app's `stars` as a pack folder, its signature, and the line of `compmd.config.json` that trusts it;
   - **In an editor**: `OpenDocument` and splices, as today.
   The header and "Where next" link to the guide.
5. **The website uses `@compmd/vite` itself**, with a `compmd.config.json` that lists its own pack folder (`content/app/`, `stars` in Svelte, trusted, ambient). The live `stars` is that pack, mounted in the page in host mode; the JSX example's frame comes from the site's own `compmd/` instead of the playground's. The site does what the guide teaches.
6. **This change writes the book's frame and its first part**, the site's pages for it, and the landing teaser. Each later part is a task of its own, so each is reviewed as it is written; the contents page lists every chapter, and those not written yet say so.

## The outline

Each chapter leads with what the reader builds, ends with what to read next, and links to the spec sections and READMEs that go deeper.

**Part I. Getting started**
1. *Introduction*: who the guide is for, what CompMD gives an application (documents people write as text, rendered as pages, safe to open from anyone), and the packages, with how a document flows through them. One figure.
2. *Quick start*: a Vite app that renders a document with an input that changes it: `npm install`, `vite.config.ts`, about thirty lines.
3. *How it fits together*: the processor's stages and trees, the renderer, the sandbox and its trust levels, the plugin and its configuration: what runs where, and why.

**Part II. Documents in your app**
4. *Processing documents*: `process` and its request, the files a document reads, diagnostics and their fixes.
5. *Live editing*: `OpenDocument`, edits and splices, hints at the caret, keeping a page in step with a text.
6. *Data and inputs*: data files, URLs (`allowedOrigins`, `sources`), the host's data (`host:`), inputs and the components that set them.
7. *Rendering and styling*: pages and regions, theme tokens and the host's stylesheets, links, what a plain renderer shows.

**Part III. Components**
8. *Components and trust*: strict, permissive and trusted, what each can reach, and who decides.
9. *Packs from npm*: installing a pack, `compmd.config.json` key by key, denying packs, limiting their size, offering libraries.
10. *Your app's own components*: a pack folder, a signature, a React or Svelte component that treats its params as untrusted input; permissive components with `host`.
11. *The sandbox, closer*: frame mode and host mode, serving `compmd/` with its two headers, editors that render block by block (`mountUse`).

**Part IV. Writing packs**
12. *Your first pack*: `npm create @compmd/pack`, the test page, the plain binding.
13. *Signatures and params*: types, defaults, slots, template slots, inputs, fallbacks.
14. *React, Svelte and libraries*: rendering a framework inside `mount`, what the frames provide, what a pack bundles.
15. *Code renderers*: rendering a language's code blocks (`--renders`), the same component as a leaf.
16. *Testing and publishing*: the component's tests, the version check, publishing with provenance, a host adding the pack.

**Part V. Beyond Vite**
17. *Other hosts*: `@compmd/vite/config` for the configuration, `buildFrames` and `buildLibraries` for the frames.
18. *The Rust crate*: processing documents outside a browser, the host traits.

**Appendices**: the configuration's reference, the diagnostics (from the spec), a glossary that links to the spec's terms.

## Questions, with proposed answers

**G1. The folder's name.** Proposed: `guide/`, after the book's name, at the repository's root beside `compmd-spec-v1.md`, and linked from the README's table. `book/` reads as any book; `docs/` holds the design documents, which are for the people who work on CompMD, not with it.

**G2. Pages.** Proposed: a page per chapter on the site and a file per chapter in the folder, numbered (`guide/02-quick-start.md`) so they sort in order, with `guide/README.md` as the contents. A chapter's address on the site drops the number (`/guide/quick-start/`), so chapters can be inserted without breaking links.

**G3. Versions.** Proposed: the guide follows `main`, as the landing page and the playground do, and names the package versions it is written for at the top of each chapter's page. A guide per release waits for version 1 of the packages, and would then be made as the spec's released versions are.

**G4. The live `stars` in Svelte or React.** Proposed: Svelte, the site's own framework, so it adds no runtime to the landing page; the guide's own example is React, the more common case, so both bindings are shown.

**G5. What this change writes.** Proposed: Part I (three chapters), the contents page with every chapter listed, the site's guide pages with the book's navigation, the snippet tests, and the landing teaser (decision 6). Parts II to V and the appendices follow as tasks of their own, in that order.

## Not in this change

- Parts II to V and the appendices (decision 6).
- A search box for the guide (the spec's search is its own task, and would serve both).
- Translations.
- API reference generated from the packages' types: the READMEs and TSDoc carry it; the guide links to them.
