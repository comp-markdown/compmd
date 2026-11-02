# The Mermaid pack

**Status: accepted (2026-11-02); built on `@compmd/pack` and the host contract's `scheme` and `fail` ([writing a pack](writing-a-pack.md)), which changed decisions 1, 4, 5 and 6.** A pack, `@comp-markdown/mermaid`, that draws Mermaid diagrams in any document that imports it, with no support from the host: the example the [code renderers](code-renderers.md) design names, and the first pack published for documents to fetch from a registry ([packs from a registry](packs-from-a-registry.md)). It is written with [`@compmd/create-pack`](create-pack.md) and follows the [registry conventions](../compmd-registry-v1.md).

## Decisions

1. **`@comp-markdown/mermaid`, in `packs/mermaid/`.** A workspace package of this repository, checked by `cargo make ci` with the rest, and released on its own (decision 9). The scope is the GitHub organization's name; `@compmd/*` is reserved and never fetched (spec §6.4). It is a pack as `npm create @compmd/pack mermaid -- --renders mermaid` writes it, on `@compmd/pack` (`workspace:*`): its build, checks and test page are `compmd-pack`'s, and it holds only its component, its signature, `demo.comp.md` and its tests.
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
4. **Mermaid draws, through `draw` from `@compmd/pack/component`.** Each draw sets `mermaid.initialize({ startOnLoad: false, securityLevel: "strict", theme })`, then `mermaid.render(id, code)`, one at a time per frame (`oneAtATime`), since Mermaid's configuration is global. Its SVG goes into a drawing with `role="img"` and `aria-label` set to `fallback.text`. Rendering is asynchronous, so a draw that an `update` overtakes is dropped: the latest params win.
5. **Light and dark follow the page.** `theme: auto` (the default) draws with Mermaid's `dark` theme when `context.scheme` is `dark`, and `default` otherwise, and draws again when it changes, which comes as an update (spec §7.4). The reader's preference alone isn't the page's, since a page may be light for a reader who prefers dark. A theme the document gives (`{theme=forest}`) is used as given.
6. **When Mermaid can't read the source**, the component fails with the first line of what Mermaid found (`context.fail("Mermaid couldn't read this diagram: …")`): the host reports W118 with it, which an editor shows on the block, and shows the fallback (the code block, or the leaf's text) until the next update, so a reader sees the source and its author the mistake, and the diagram draws once the code is fixed (spec §7.4).
7. **One module file, under the size cap.** `dist/mermaid.js` holds Mermaid and everything it loads, with code splitting off (as `create-pack` builds a pack), so a host can load it by URL or as text. Its size, minified and gzipped, is measured and written in the README. It must stay under `@compmd/registry`'s default cap, which is **10 MB**: measured on 2026-11-02, Mermaid 12.2.0 as one file is 7.0 MB minified (1.6 MB gzipped), 2.5 MB of it the ELK layout engine (`elkjs`), which Mermaid 12 bundles and 11 didn't; Mermaid 11.17.2 is 4.58 MB (1.03 MB gzipped). The cap was 5 MB, from Mermaid 11.12's 3.6 MB, and was raised so that Mermaid 12 can be taken whole.
8. **The licences of what it bundles are published with it.** Mermaid is MIT, and it bundles libraries under their own licences (DOMPurify, d3, dagre, cytoscape, KaTeX…). The build writes them into `dist/LICENSES.md` (Vite's `build.license`), which the pack publishes beside its own `LICENSE`.
9. **Released on its own tag.** The implementation's release workflow publishes the `@compmd` packages at one version on a tag `v<version>` ([releases](releases.md)), which a pack with its own version doesn't fit. A tag `mermaid-v<version>` runs a workflow of its own, `release-mermaid.yml`: it checks the tag is `packs/mermaid/package.json`'s version, builds and tests the pack, and stages it with trusted publishing, installable once the author approves it ([releases](releases.md), decision 5). Its first version is published by hand, as the `@compmd` packages' were (`RELEASING.md`, prerequisites), once the `@comp-markdown` organization exists on npm; every publish needs the author's go.

## What it runs into in the strict sandbox

The strict frame's policy (`@compmd/sandbox`'s `policy.ts`) allows scripts only from the frames' origin and `blob:`, inline styles, and no network, workers or `eval`.

| Mermaid needs | In the strict frame |
|---|---|
| Its own `<style>` in each SVG | allowed: `style-src 'unsafe-inline'` |
| No network: fonts, icons | none by default: its fonts are system fonts, and icon packs load only when registered |
| No `eval` or `new Function` | to check in the browser tests: the policy has no `'unsafe-eval'`, so a diagram type that needs it fails there, and is listed |
| No workers | none |
| Measuring text (`getBBox`) | a real layout: works in a frame, not in happy-dom |

## What building it found

Three things in `@compmd/render` and `@compmd/sandbox`, fixed with the pack:

- **A rendered code block lost its fallback in host mode.** The page gave the code block's frame the renderer's leaf, which has neither the block's `alt` nor its code, so the component's `fallback.text` was empty and its `fallback.element` an empty box, against spec §7.7. The frame now gets the code block itself (`part {node}`), and renders it as the page would.
- **Frames had no colour scheme.** A frame's document doesn't inherit its page's, so on a dark page each frame was an opaque white box, whatever its components drew, and a component couldn't tell a light page from a dark one. The sandbox now reads the page's scheme from the container it renders in (its `color-scheme`, and the reader's preference where it allows both), sends it to every frame (`scheme {scheme}`, [the protocol](sandbox-protocol.md)), and again when it changes; each frame sets it on its document and passes it to its own frames.
- **A frame was sized without its content's margins.** A frame reports its body's height, which left out margins that collapse through the body, so a frame whose content ended with a margin scrolled. The frame's body now holds them (`display: flow-root`).

Its review then found that it carried routine code (a Vite configuration, a publish check, a test page) and repeated what every asynchronous component does (the newest draw winning, finding the page's scheme, a failure shown late): [writing a pack](writing-a-pack.md) moved those into `@compmd/pack` and the host contract, and the pack was rewritten on them.

## Tests

- **In Chrome and WebKit**, the test page `compmd-pack dev` serves (the module loaded by URL, in host mode, each diagram in a strict frame of its own that fits it), in light and dark (Playwright's `colorScheme`, the page `color-scheme: light dark`) and switched by the page's own control: a flowchart block, a sequence diagram block, a leaf computed from data, a diagram laid out with ELK, a block with a syntax error (its code, and W118 with what Mermaid found), and `theme=forest`; no frame scrolls, and the frames take the page's dark scheme.
- **As a registry pack**, in Chrome: the packed tarball served by a local registry, resolved by `resolvePacks` in a page, its module given to the sandbox as code (`{ code }`, a `blob:` module), and a diagram drawn.
- **Unit tests** (with `mountForTest`, Mermaid stubbed): the theme `auto` chooses for each scheme, and the error path, failing until the code is fixed. The latest draw winning is `draw`'s, tested in `@compmd/pack`.
- The built module: one file, its size under the cap, `dist/LICENSES.md` there.

## Not in this change

- The playground document that imports the pack from npm: the playground's own change, once the pack is published.
- Diagram types the strict policy can't run, if the tests find any: listed in the README, not worked around.
