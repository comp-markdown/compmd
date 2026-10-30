# The CompMD website

The landing page, with the playground and the spec, assembled into one static site and published on GitHub Pages at [comp-markdown.github.io/compmd](https://comp-markdown.github.io/compmd/). Why it is built this way is in [the design](docs/design.md).

| Address | What | Made from |
|---|---|---|
| `/` | The landing page | `src/App.svelte`, with the documents in `content/` |
| `/spec/` | The spec: the draft, then the latest release | `compmd-spec-v1.md`, and each `spec-v*` tag |
| `/playground/` | The playground | `editor/`'s build |
| `/logo/` | The logo files | `logo/` |

## How it is built

- **The landing page** is Svelte, built by Vite, rendered on the server at build time and hydrated in the browser, so it paints from its HTML before any script runs, with its stylesheet inlined.
- **Every example is processed by the processor.** The documents in `content/` are processed and rendered when the page is built (`build/snapshots.ts`, served to the page as `virtual:snapshots`), and the page shows those snapshots until the processor loads. Then the hero is live (`src/live.ts`): its text can be edited, and the page follows. The example with an input is live in the same way. The example with a JSX component renders in the playground's sandboxed frame (`src/framed.ts`).
- **The contents wheel** (`src/SectionWheel.svelte`) shows the section being read, and turns as the page scrolls.
- **The spec page** is the spec's Markdown as plain GFM HTML (the processor's `plainHtml`), with the code highlighted by [Shiki](https://shiki.style) at build time (`build/highlight.ts`, with grammars of our own for CompMD's syntax and ABNF in `build/grammars.ts`). It has ids on the headings as GitHub gives them, a link to each heading's source, and the requirement words marked (`build/spec.ts`). It is static HTML: `src/spec.ts` only adds the scroll-spy and the phone menu. Each version goes in a folder of its own (`build/versions.ts`).
- **The snippets** of "Use it in your app" are the files in `snippets/`, which tests run as they are written.
- **Visits are counted** with [GoatCounter](https://www.goatcounter.com), without cookies (`src/count.ts`). Its script loads once a page has loaded, on the landing page and the spec page, never in the playground's frame. It counts only on the published site: served from this machine, as by `pnpm dev` and `pnpm check`, it doesn't load.

`pnpm build` runs:
1. `vite build`: the two pages' scripts and styles.
2. The server build of the pages.
3. `scripts/prerender.ts`: writes the rendered pages, the landing page and each version of the spec, into `dist/`.
4. `scripts/assemble.ts`: copies the playground and the logo in.

## Commands

They need the packages built first: `pnpm -r build` from the root builds them all, this site last.

```sh
pnpm dev       # the dev server, at http://localhost:5173/; the spec page renders from the working tree, and the playground is editor/'s last build
pnpm build     # the site, in dist/
pnpm preview   # dist/ served as GitHub Pages serves it, at http://localhost:4173/
pnpm check     # the built site checked in Chrome: every page, light and dark, desktop and phone; the live parts; the links; the spec's contents; the playground's documents; the budgets
pnpm lint      # eslint, tsc and svelte-check
pnpm test      # the snapshots, the spec's rendering and highlighting, the versions, the prerender, the snippets
```

## Budgets

`pnpm check` measures them on the built site:

- **Size:** the landing page before its live parts is at most 100 KB gzipped. It is 37 KB.
- **Paint:** the largest paint is within 1 s on a mid-range phone over fast 3G (Chrome, CPU slowed 4×, 562 ms round trips, 1.47 Mbps; median of 5). It is 0.77 s.

Measured on 2026-10-30 on an Apple M4. Without the prerender, the page painted at 1.77 s, since nothing showed until its script had run.

## Deploying

`.github/workflows/pages.yml` builds the site, checks it in the runner's Chrome (`pnpm check --no-timing`, since a shared runner's timings vary too much to hold the paint budget), and deploys it to GitHub Pages. It runs:
- on every push to `main` that touches what the site is built from;
- on every `spec-v*` tag;
- by hand.

It needs the repository's Pages source set to "GitHub Actions" (Settings, Pages). For the site's own address, `compmd.hakanserce.com`:
1. A DNS `CNAME` record from `compmd.hakanserce.com` to `comp-markdown.github.io`.
2. The domain verified for the `comp-markdown` organization (its settings, Pages), so no one else's Pages can claim it.
3. The custom domain set in the repository's Pages settings, with HTTPS enforced once GitHub has its certificate.
