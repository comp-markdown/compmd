# The CompMD website

**Status: accepted.**

CompMD has a spec, a playground and packages, but no front door: someone who hears of it lands on a GitHub README. The website is that door: one page that shows what CompMD is and why it exists, in a way that makes someone want to try it, and that leads on to the three things they come for: **the spec**, **the playground**, and **how to use CompMD in their own application**. It is published with the playground, from this repository.

## Decisions

1. **One site, three parts, from this repository.** `website/` holds the landing page and this document; a build puts the landing page at `/`, the playground at `/playground/` and the spec at `/spec/`. Each part keeps its own folder and build (the playground stays `editor/`); `website/` only assembles them.
2. **Hosted on GitHub Pages, deployed by a GitHub Actions workflow** in `comp-markdown/compmd`, on every push to `main` that touches what the site is built from, on every release tag, and by hand. Pushes update the drafts; tags add released versions of the spec, which never change (decision 11). The playground's About names the commit it was built from. A static host is all the site needs: the playground makes no requests once loaded, and GitHub Pages sends `Access-Control-Allow-Origin: *`, which the playground's frame needs to load the site's scripts from its opaque origin (to check on the first deploy, with the frame's content security policy).
3. **The address comes later.** The site first goes up at GitHub's own address, `comp-markdown.github.io/compmd/`, and gets a name of its own later: a `compmd` subdomain of a domain the author already has (`hakanserce.com` or `vuesical.com`), as a `CNAME` record to `comp-markdown.github.io`. Every page links with relative paths, as the playground already does, so the site works under either address, and the move is a `CNAME` file and a DNS record.
4. **The landing page is one long page, built with Vite and Svelte** as the playground is, sharing its look (the logo, the ink, the one blue accent, light and dark) but louder: it is the one place CompMD sells itself. It is a static page first: the text, the layout and a snapshot of the live demo are HTML at build time, so it paints at once and reads without JavaScript; the live parts load after.
5. **The hero is the real thing.** A short CompMD document and the page it renders, side by side, and the reader can type in it: the processor (WebAssembly, 758 KB gzipped) and the renderer load once the page has painted, and until then the hero shows the same document as a snapshot made at build time. Nothing on the page is a mock-up of CompMD; every example is processed by the processor.
6. **The spec is a page of the site, made for reading a long document**, `/spec/`, rendered from `compmd-spec-v1.md` at build time (GFM, through the processor's `plainHtml`):
   - a table of contents beside the text, every section and subsection, that marks where the reader is as they scroll, folds to a menu on a phone, and links each heading;
   - code highlighted at build time, so the page runs no script for it: YAML, JavaScript, TypeScript, ABNF and the Markdown examples, with CompMD's own syntax (`:::`, `{params}`, `{{paths}}`) marked in them;
   - the tables, notes and the rules (MUST, SHOULD) set to be scanned; a link to the source of each section on GitHub; light and dark;
   - the draft and each released version at addresses of their own, each linking to the others (decision 11).

   The highlighting is [Shiki](https://shiki.style)'s, run at build time: VS Code's highlighting, its output plain HTML with no script. It is a new dependency, of the site's build only, with a grammar of our own for CompMD's syntax.
7. **"Use it in your app" shows real code that runs, and as little of it as can be.** One install line, `npm install @compmd/processor @compmd/render`, and three short snippets, each a tab: render a document in a page; keep it open in an editor (`OpenDocument`, splices); and run components written in React or Svelte. The React and Svelte bindings are part of `@compmd/render` (`@compmd/render/react`, `@compmd/render/svelte`, with React and Svelte as optional peer dependencies), so no third package is needed. Each snippet is checked by a test that runs it, so it can't go stale. It needs the two packages published on npm; until then the install line says so and links to their folders on GitHub. Once they are, the install line and the snippets are those of the latest release ([how releases work](../../docs/releases.md)).
8. **Visits are counted, not readers tracked**, with [GoatCounter](https://www.goatcounter.com): open source, free for a site like this, without cookies, counting page views, referrers, browsers and countries and nothing that identifies a reader, so the site needs no consent banner. Its script is the only one from elsewhere; the playground's frame stays without network, as spec §15 asks of components. The account is the author's.
9. **"Under the hood": how it is built**, for the developers who'll judge whether to use it: the processor in Rust, run in the browser as WebAssembly; incremental processing, so a keystroke in a long document is processed in a fraction of a millisecond (with the measured numbers); diagnostics with fixes, and hints at the caret; the CEL profile implemented from scratch, type-checked before it runs and evaluated within a work budget, with CompMD's `md.` functions; and the conformance set any implementation can run. Each point links to where it is documented.
10. **Security has a section of its own: "Safe to open".** A CompMD document can come from anyone, and the page says plainly what opening one can and can't do, and why:
    - **The threat:** a document from a stranger, or a pack it imports, trying to run code in the reader's page, read the reader's data or session, send data out, run forever, or act in the reader's name.
    - **What stops it:** `compmd` definitions run no code at all, only Markdown, params and slots; code in JSX, Svelte or JavaScript runs sandboxed, seeing only its params and slots, with no network, no storage or cookies, no navigation, and stopped when it runs too long; data comes only from origins the reader or the host allows, never ones a document grants itself; text is escaped, and raw HTML is limited to a short list of safe tags; expressions run within a work budget, and expansion within a size limit, so no document can hang the page; components a host trusts with more act outside the document only on the reader's action, after showing what they'll do.
    - **How it is built in this implementation:** the playground's frame (a sandboxed iframe with an opaque origin and a content security policy that allows only its own files), the expression budget, the expansion limit, the checks that refuse recursion.

    Each point links to the spec's section 15 and the code that enforces it.
11. **Drafts follow `main`; releases are frozen**, as W3C publishes a spec's Editor's Draft from its repository beside its dated versions, and CommonMark its `dev` spec beside its numbered ones:
    - Until version 1 is released, `/spec/` is the draft from `main`, with a "Draft" banner.
    - A release tag (`spec-v1.0`) adds `/spec/1.0/`, the spec as it was at that tag, never changed after; `/spec/` then shows the latest release, and `/spec/draft/` the draft.
    - Pages replaces the whole site on every deploy, so each build makes every released version again, from the spec at its tag (`git show spec-v1.0:compmd-spec-v1.md`): reproducible, with no old build kept anywhere.
    - The landing page and the playground follow `main`.

## The landing page

One column of sections, each a single idea with something to look at; nothing hidden behind a click. In order:

```text
┌──────────────────────────────────────────────────────────────┐
│ [logo] CompMD                      Spec  Playground  GitHub  │  header, sticky, thin
├──────────────────────────────────────────────────────────────┤
│ Markdown with components.                                    │  hero: one sentence, the
│ Readable as text, rendered as a page.                        │  demo, two buttons
│ ┌──────── report.comp.md ───────┐ ┌────── the page ───────┐  │
│ │ ::: grid {min=14rem gap=m}    │ │ ┌──────┐ ┌──────┐     │  │
│ │ :: stat[Revenue]{value=…}     │ │ │ $48k │ │ +12% │     │  │
│ │ ...                           │ │ └──────┘ └──────┘     │  │
│ └───────────────────────────────┘ └───────────────────────┘  │
│ [Open the playground]  [Read the spec]                       │
├──────────────────────────────────────────────────────────────┤
│ Why: Markdown stops where pages start — and the fixes leave  │  why: the gap, in three
│ Markdown behind (MDX is code; templates hide the text).      │  short beats
├──────────────────────────────────────────────────────────────┤
│ What: five things, each source beside its page               │  what: components, slots,
│  components · data · inputs · layout · your own components   │  data, inputs, layout,
│                                                              │  host components
├──────────────────────────────────────────────────────────────┤
│ Still Markdown: the same file in a plain renderer            │  degradation (spec §14):
│  (GitHub, an editor) reads fine — the fallbacks              │  a toggle, CompMD / plain
├──────────────────────────────────────────────────────────────┤
│ Great for plain Markdown too: no components needed           │  plain Markdown, rendered
├──────────────────────────────────────────────────────────────┤
│ Use it in your app: [Page] [Editor] [React/Svelte]  + code   │  snippets, install line
├──────────────────────────────────────────────────────────────┤
│ Safe to open: a document from anyone can't run code in       │  security: the threat,
│ your page, reach your data, or call home                     │  what stops it, how
├──────────────────────────────────────────────────────────────┤
│ Under the hood: Rust · WebAssembly · incremental · CEL ·     │  how it is built, with
│ diagnostics and hints · conformance                          │  numbers and links
├──────────────────────────────────────────────────────────────┤
│ Where next: Spec · Playground · Conformance · GitHub · npm   │  the portal, as cards
├──────────────────────────────────────────────────────────────┤
│ MIT / CC BY 4.0 · Copyright © 2026 Hakan Serce               │  footer
└──────────────────────────────────────────────────────────────┘
```

- **Hero.** One line that says what it is, one that says why it's different; the live document beside its page; two buttons. The document is a small report or note that shows four things at a glance: a grid of components, a value from data, an input that changes what the page shows, and plain prose around them.
- **Why.** The problem in three beats, each a sentence and a small figure: Markdown can't place a chart or a card; the usual way out (MDX, templates) turns the text into code; CompMD adds components that read as text and degrade to plain Markdown.
- **What.** One row per feature (components and slots, data, inputs, layout, components of your own in JSX or Svelte), each a few lines of source beside what they render, processed by the processor at build time.
- **Still Markdown.** The same document twice, switched by a toggle: as CompMD renders it, and as GitHub or any Markdown renderer shows it, fallbacks and all (spec §14). This is CompMD's strongest point, and one the formats it is compared with (`existing-specs-comparison.md`) mostly give up, so it gets room: a file written in CompMD stays a good Markdown file for every tool that already reads Markdown.
- **Great for plain Markdown too.** A document with no components at all is a CompMD document, and gets what the processor and the renderer give every document: CommonMark and GFM as GitHub renders them, with alerts and math; a live preview that renders only the blocks an edit changed, and keeps the rest of the page as it was; every rendered block knowing the lines it came from, so a click on the page finds its source; links to a passage of the text; and diagnostics for broken links, footnotes and frontmatter. Each claim is checked against the processors the comparison covers before it goes on the page, and written as what CompMD does, not as what others don't.
- **Use it in your app.** The install line and the snippets of decision 7.
- **Safe to open.** Decision 10's three parts, as three columns on a wide screen: the threat, what stops it, how this implementation builds it. A small figure of the sandbox: the page, the frame inside it, and the component inside the frame with nothing leading out.
- **Under the hood.** The points of decision 9, each a short paragraph with a number or a figure, and a link to its documentation.
- **Where next.** The spec, the playground, the conformance set, GitHub, and npm, each a card with one line on what it is for.

**Look.** The playground's palette (ink `#1b2430`, accent `#3557c4`, cool paper, and their dark counterparts) and system fonts, so nothing loads to show the page; the logo's rounded strokes and its corner tile as the one motif, used sparingly (the tile marks the live parts). One moment of motion: the hero's page assembling as its document is typed, once, on load, and never for a reader who asks for less motion. Responsive to a phone, where the hero stacks the document over the page.

**Budgets.** The page before the live parts: at most 100 KB gzipped, painted within one second on a mid-range phone over a fast 3G profile. The live parts load after, and only on a screen that shows them. Measured in Chrome with Playwright, as the playground's are.

## The site

```text
/                  the landing page                     website/
/playground/       the playground                       editor/ (its dist/)
/spec/             the spec: the draft, then the latest release   compmd-spec-v1.md
/spec/draft/       the draft, once a version is released          compmd-spec-v1.md on main
/spec/1.0/         a released version, frozen                     compmd-spec-v1.md at its tag
/logo/             the logo files                       logo/
```

- `website/` is a pnpm workspace member with `pnpm build`, which builds the landing page, renders the spec, and assembles the site with the playground's `dist/` into `website/dist/`; `pnpm preview` serves it as Pages will, `Access-Control-Allow-Origin: *` included.
- **The workflow** (`.github/workflows/pages.yml`): the Rust toolchain of `rust-toolchain.toml` with the `wasm32-unknown-unknown` target, `wasm-bindgen-cli` at the version `Cargo.toml` pins, pnpm; then `pnpm -r build` and the site's build, and GitHub's Pages actions to upload and deploy. It checks out every tag, for the released versions of the spec. It shares its setup with the CI workflow ([`docs/ci.md`](../../docs/ci.md)).
- **The domain:** a `CNAME` file in the site, the DNS record (a `CNAME` to `comp-markdown.github.io` for a subdomain; GitHub's `A` and `AAAA` records for an apex domain), and HTTPS enforced in the repository's Pages settings; the domain verified for the `comp-markdown` organization, so no one else's Pages can claim it.
- **Links:** the CompMD README, the playground's About and header, and the spec's top link to the site; the site links back to the repository.

## Tests

- A Playwright check of the built site in Chrome, light and dark, at desktop and phone widths: the landing page's sections, the hero typing and rendering, the plain toggle, every link resolving within the site, the playground opening every document of its menu, the spec's table of contents.
- The snippets run as tests against the packages, as the packages' own tests do.
- The budgets, measured from the built site.

## Not in this change

- Publishing the npm packages and the crate (the `@compmd` scope on npm, versions, the crate's dependency on the fork of markdown-rs).
- The landing page written in CompMD itself, rendered by its own processor, so the page is its own example: after this Svelte version, which goes up first. It needs the renderer to take over HTML rendered at build time.
- A search of the spec, a blog, translations.
- The CI workflow ([`docs/ci.md`](../../docs/ci.md)).
