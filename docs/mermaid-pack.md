# The Mermaid pack

**Status: proposed.** A pack, `@comp-markdown/mermaid`, that draws Mermaid diagrams in any document that imports it, with no support from the host: the example the [code renderers](code-renderers.md) design names, and the first pack published for documents to fetch from a registry ([packs from a registry](packs-from-a-registry.md)). It is written with [`@compmd/create-pack`](create-pack.md) and follows the [registry conventions](../compmd-registry-v1.md).

## Decisions

1. **`@comp-markdown/mermaid`, in `packs/mermaid/`.** A workspace package of this repository, checked by `cargo make ci` with the rest, and released on its own (decision 9). The scope is the GitHub organization's name; `@compmd/*` is reserved and never fetched (spec §6.4). It starts as `npm create @compmd/pack mermaid -- --renders mermaid` writes it, with `workspace:*` for the `@compmd` packages it builds and tests with.
2. **Its own version, from `0.1.0`**, for its component's API: its params and both forms. It isn't a library pack, so it doesn't take Mermaid's version. It bundles **Mermaid 12.2.0** (2026-10-13), pinned exactly, and its README and `description` say which Mermaid it bundles. A new Mermaid is a new version of the pack: a minor one for a new Mermaid minor, a patch for a Mermaid patch.
3. **One component, `mermaid`, in both forms** (spec §7.7):

   ````md
   ```compmd component=mermaid signature
   renders: [mermaid, mmd]
   params:
     theme: auto | default | dark | neutral | forest = auto
   ```
   ````

   A ` ```mermaid ` block, its text the `code` param, with `alt` as its text alternative; and `:: mermaid[…] {code=(…)}` for a diagram computed from data. A plain renderer shows the fence, and GitHub draws it.
4. **Mermaid draws, and the component places the drawing.** `mermaid.initialize({ startOnLoad: false, securityLevel: "strict", theme })` once per frame, then `mermaid.render(id, code)` for each draw, whose SVG goes into `target` with `role="img"` and `aria-label` set to `fallback.text`. Rendering is asynchronous, so a draw that an `update` overtakes is dropped: the latest params win.
5. **Light and dark follow the reader.** `theme: auto` (the default) draws with Mermaid's `dark` theme when the frame's colour scheme is dark, and `default` otherwise, and draws again when it changes. The frame's colour scheme is the reader's (`prefers-color-scheme`), as the page's is when the page follows it, which the playground and the website do. A theme the document gives (`{theme=forest}`) is used as given.
6. **When Mermaid can't read the source**, `target` shows the fallback (`fallback.element`: the code block, or the leaf's text), with one line under it saying what Mermaid found (`Mermaid couldn't read this diagram: …`), so a reader sees the source and its author the mistake. Nothing is thrown: `mount` is synchronous and Mermaid answers later, so W118 can't be reported (spec §7.4).
7. **One module file, under the size cap.** `dist/mermaid.js` holds Mermaid and everything it loads, with code splitting off (as `create-pack` builds a pack), so a host can load it by URL or as text. Its size, minified and gzipped, is measured and written in the README. It must stay under `@compmd/registry`'s 5 MB default cap: the design measured Mermaid 11 at 3.6 MB as one file.
8. **The licences of what it bundles are published with it.** Mermaid is MIT, and it bundles libraries under their own licences (DOMPurify, d3, dagre, cytoscape, KaTeX…). The build writes them into `dist/LICENSES.md` (Vite's `build.license`), which the pack publishes beside its own `LICENSE`.
9. **Released on its own tag.** The implementation's release workflow publishes the `@compmd` packages at one version on a tag `v<version>` ([releases](releases.md)), which a pack with its own version doesn't fit. A tag `mermaid-v<version>` runs a workflow of its own, `release-mermaid.yml`: it checks the tag is `packs/mermaid/package.json`'s version, builds and tests the pack, and publishes it with trusted publishing. Its first version is published by hand, as the `@compmd` packages' were (`RELEASING.md`, prerequisites), once the `@comp-markdown` organization exists on npm; every publish needs the author's go.

## What it runs into in the strict sandbox

The strict frame's policy (`@compmd/sandbox`'s `policy.ts`) allows scripts only from the frames' origin and `blob:`, inline styles, and no network, workers or `eval`.

| Mermaid needs | In the strict frame |
|---|---|
| Its own `<style>` in each SVG | allowed: `style-src 'unsafe-inline'` |
| No network: fonts, icons | none by default: its fonts are system fonts, and icon packs load only when registered |
| No `eval` or `new Function` | to check in the browser tests: the policy has no `'unsafe-eval'`, so a diagram type that needs it fails there, and is listed |
| No workers | none |
| Measuring text (`getBBox`) | a real layout: works in a frame, not in happy-dom |

## Tests

- **In Chrome and WebKit**, the pack's test page (`npm run dev`'s, on `@compmd/vite`, the module loaded by URL), in light and dark (Playwright's `colorScheme`): a flowchart block, a sequence diagram block, a leaf computed from data, a block with a syntax error (its fallback and the note), and `theme=forest`.
- **As a registry pack**, in Chrome: the packed tarball served by a local registry, resolved by `resolvePacks` in a page, its module given to the sandbox as code (`{ code }`, a `blob:` module), and a diagram drawn.
- **Unit tests** (happy-dom, Mermaid stubbed): the theme `auto` chooses, the latest draw winning over an earlier one, and the error path.
- The built module: one file, its size under the cap, `dist/LICENSES.md` there.

## Not in this change

- The playground document that imports the pack from npm: the playground's own change, once the pack is published.
- A colour scheme from a page that has its own light and dark switch, not the reader's: the sandbox sends a page's `--compmd-*` tokens to frames, but no colour scheme. That is a change to the sandbox, for later.
- Diagram types the strict policy can't run, if the tests find any: listed in the README, not worked around.
