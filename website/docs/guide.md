# The CompMD guides

**Status: accepted (2026-10-31).** Two books, for the two kinds of reader: *the CompMD Author's Guide*, for people who write documents, and *the CompMD Developer's Guide*, for developers who put CompMD in an application or write packs for it. Their chapters are in the repository, their pages on [the website](design.md) at `/guides/`, and the landing page teases the Developer's Guide. The spec says what CompMD *is*; the package READMEs say what each package *has*; the Author's Guide teaches how to *write* in it, and the Developer's Guide how to *build with* the packages, from a first page to a published pack.

## Proposed decisions

1. **The guides are books in a folder of their own, `guides/`**, beside the spec: a shelf page (`guides/README.md`) that introduces both, and a folder per book, `guides/authors/` and `guides/developers/`, each with a contents page (`README.md`) and one Markdown file per chapter, grouped in parts. They read on GitHub as books do, change with the code in the same commits, and grow a chapter at a time. A book links to the other where their readers meet: a component in CompMD is the Author's Guide's, the same component in code the Developer's.
2. **`guides/` is a workspace package**, so its code is real: each example is a folder in its book's `examples/`, its code and its CompMD documents together, type-checked and run by the package's tests against the repository's packages. A chapter writes its code inline, so it reads anywhere, and a test checks that each block marked with a file's name (` ```ts snippet=quick-start/main.ts `) is that file's text: the guides can't drift from code that works.
3. **On the site, the guides are pages of their own: `/guides/`, `/guides/<book>/` and `/guides/<book>/<chapter>/`**, made at build time with the spec page's renderer (the website's decision 6): the processor's `plainHtml`, code highlighted by Shiki with CompMD's grammar, headings linked. Each page has both books beside the text, its own open to its chapters, its own sections under the current chapter, and links to the chapters before and after it. `/guides/` is the shelf, `/guides/<book>/` a book's contents.
4. **The landing page teases the Developer's Guide.** "Use it in your app" becomes three tabs of short code from the guide's snippets, the install line, a live `stars`, and a button to the guide:
   - **In a Vite app**: `vite.config.ts` with `compmd()`, and a document processed and rendered with `virtual:compmd`;
   - **Your own components**: the app's `stars` as a pack folder, its signature, and the line of `compmd.config.json` that trusts it;
   - **In an editor**: `OpenDocument` and splices, as today.
   The header and "Where next" link to the shelf.
5. **The website uses `@compmd/vite` itself**, with a `compmd.config.json` that lists its own pack folder (`content/app/`, `stars` in Svelte, trusted, ambient). The live `stars` is that pack, mounted in the page in host mode; the JSX example's frame comes from the site's own `compmd/` instead of the playground's. The site does what the guide teaches.
6. **An example in a guide opens where it can be played with.** A chapter's CompMD example is a document in its example's folder, and a **Try it** link under it opens it in the playground (`/playground/?doc=guides/<book>/<name>`), which bundles the guides' examples beside its menu. When the playground's tools become the *Developer Tools* (their own design: the text, the page and the views as a part any page can embed, with tabs for the sandbox's frames, a console and the messages), the same links open there.
7. **This change writes the shelf, both books' contents, the Developer's Guide's first part**, the site's pages for them, and the landing teaser. Each later part of either book is a change of its own, so each is reviewed as it is written; a contents page lists every chapter, and those not written yet say so.
8. **Each book downloads as a PDF** (added 2026-11-02), from a link in the status line of its contents and of every chapter: one page of the whole book, its contents and then each chapter, printed by Chrome at every build of the site, with an outline from the headings and page numbers at the foot. The spec has one too, for each of its versions, printed from its page with its companion. Chrome keeps the site's look; a reader of CompMD documents for pandoc, which would give book typography and page numbers in the contents, is its own task.

## The outlines

Each chapter leads with what the reader builds, ends with what to read next, and links to the spec sections and READMEs that go deeper. The Author's Guide's outline is its contents page, [`guides/authors/README.md`](../../guides/authors/README.md): writing, data and inputs, layout, components of your own, and packs, for writers, with no code. The Developer's Guide's:

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

**G1. The folder's name.** Proposed: `guides/`, at the repository's root beside `compmd-spec-v1.md`, and linked from the README's table. `book/` reads as any book; `docs/` holds the design documents, which are for the people who work on CompMD, not with it.

**G2. Pages.** Proposed: a page per chapter on the site and a file per chapter in its book's folder, numbered (`guides/developers/02-quick-start.md`) so they sort in order, with the book's `README.md` as its contents. A chapter's address on the site drops the number (`/guides/developers/quick-start/`), so chapters can be inserted without breaking links.

**G3. Versions.** Proposed: the guide follows `main`, as the landing page and the playground do, and names the package versions it is written for at the top of each chapter's page. A guide per release waits for version 1 of the packages, and would then be made as the spec's released versions are.

**G4. The live `stars` in Svelte or React.** Proposed: Svelte, the site's own framework, so it adds no runtime to the landing page; the guide's own example is React, the more common case, so both bindings are shown.

**G5. What this change writes.** Proposed: Part I (three chapters), the contents page with every chapter listed, the site's guide pages with the book's navigation, the snippet tests, the Try it links, and the landing teaser (decision 7). Parts II to V and the appendices follow as tasks of their own, in that order.

## Not in this change

- The Author's Guide's chapters, and the Developer's Guide's Parts II to V and appendices (decision 7).
- The Developer Tools (decision 6): their own design, then their own tasks.
- A search box for the guide (the spec's search is its own task, and would serve both).
- Translations.
- API reference generated from the packages' types: the READMEs and TSDoc carry it; the guide links to them.
