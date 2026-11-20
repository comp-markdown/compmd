# Spike: four npm packages wrapped as packs (CMD-8)

**Status: done (2026-11-20).** [Wrapping an npm package as a pack](../../docs/wrapping-an-npm-package.md) proposes `npm create @compmd/pack <folder> -- --from <package>`. The command doesn't exist yet, so this spike writes by hand, for four packages, the packs it would write, builds them as `compmd-pack build` does, and opens each demo in Chrome and WebKit as `compmd-pack dev` serves it, each use in a strict frame of its own. Nothing here changes the packages; what it changes in the design is listed first.

## What it found

1. **`package.json`'s `compmd` field is taken.** The registry companion (section 2) makes it the pack's document (`"./index.comp.md"`) or a library pack's module (`{ "library": … }`), so the design's `compmd.wrap` can't go there. The spike keeps the wrap's settings in `compmd.wrap.json` beside it, which the build reads and the pack doesn't publish.
2. **A prop named like a common param is the box's.** `width`, `height`, `background`, `border`, `padding` and the rest of spec §9.2 style the use's box, the component's `target`, and never reach the component: layerchart's `height` was dropped, even when the use gave one. A chart that takes its size from its container fills `target`, and a document sizes it with `height=12rem`. The mapping leaves such props out, and lists them.
3. **CSS colours map to `color`**, the spec's type for CSS colour syntax (§9.4), not `string`: react-qr-code's `bgColor` and `fgColor`.
4. **"A prop's own declaration" means one in the package's own files.** layerchart's `x` and `y` come from its `ChartProps`, which `BarChartProps` joins with `&`; React's `SVGProps` (react-qr-code) is what is left out.
5. **The glue mounts in a box of its own.** The renderer owns `target`'s classes and style, so a style the glue sets on `target` may not hold; layerchart's chart is mounted in a `div` the glue adds.
6. **A web component's failure is an event.** model-viewer reports a model it couldn't load with an `error` event and throws nothing, so without the glue passing it to `fail` the reader sees an empty box and the host gets no W118. The glue the command writes for a web component listens for `error`.
7. **A Svelte package's CSS goes into the module.** Compiled with Svelte's default, the CSS went to a file of its own, which the frame never loads, and the chart drew unstyled; compiled with `css: "injected"`, the module adds it. Its theme's colours (`--color-primary`) are still unset: the bars are `currentColor` (the design's gap G3).
8. **Svelte's props can't change from a plain module:** `$state` needs a `.svelte` file, so the glue mounts the chart again on each update.
9. **An optional enum works:** `level: L | M | H | Q?` is accepted, and `level=H` checked (the design's gap G6, which still has no conformance case).
10. **`generated` after `signature` in the fence's info string,** the first way the spike marked the build's fence, is ignored by the processor, but the spec doesn't say what other words there mean. The design settled on comments before and after the fence instead (decision 4), which the packs here use.

## The packs

Each folder is what the command would write, with the glue as written once, and the demo it would write a use of each param into. The signatures between `compmd-pack:generated` comments are written by hand as the build would write them from the package's types.

| Folder | Package | Kind | Built (gzipped) | Imports left | Drew, Chrome and WebKit | What the frame refused |
|---|---|---|---|---|---|---|
| [`react-qr-code/`](react-qr-code/) | `react-qr-code` 2.2.0 | React | 8.9 KB | `react`, `react-dom/client` | 2 of 2 | nothing |
| [`layerchart/`](layerchart/) | `layerchart` 2.11.0, `BarChart` | Svelte 5, `.svelte` sources | 164.5 KB | nine of `svelte`'s | 2 of 2 | nothing |
| [`model-viewer/`](model-viewer/) | `@google/model-viewer` 4.4.0, with `three` 0.183.2 | web component | 289.3 KB | none | 0 of 1: its fallback, with W118 | the model's fetch (`connect-src`), and WebAssembly (`wasm-unsafe-eval`) |
| [`qrcode/`](qrcode/) | `qrcode` 1.5.4 | library, CommonJS | 9.3 KB | none | 2 of 2 | nothing |

- **react-qr-code:** six params from its six own props; `size`'s default, 256, is the author's (its types' comment says 128). React and React DOM stay imports, which the frames' import map gives.
- **layerchart:** five params of about 90, the allowlist; `x` and `y` as field names, their `string` branch. Its `.svelte` sources compile with the frames' Svelte, and d3 and its other dependencies are bundled.
- **model-viewer:** the strict frame refuses what the design expected, and the page shows the label in its place. Its decoders from a CDN never came up: the model didn't load.
- **qrcode:** nothing derives; the author wrote the call, `toString(value, { type: "svg" })`, and the signature, which has no `compmd-pack:generated` comments. Its SVG takes the frame's width; `width=10rem` on the use sizes it.

`results.json` has each run's numbers, and `screenshots/` each page.

## Running it

```sh
pnpm install
pnpm --dir spikes/wrap-npm-package try                 # every pack
pnpm --dir spikes/wrap-npm-package try layerchart      # one
```

`try.ts` builds each pack with `compmd-pack build`'s own configuration (with Svelte's Vite plugin for a Svelte package, which the build doesn't have yet), serves its `demo.comp.md` as `compmd-pack dev` does, and opens it in Playwright's Chrome and WebKit (1.64.1, headless). A use counts as drawn when its frame has a height and its `svg` or `canvas` a size. The packs have no `build` or `test` script, so the workspace's runs leave them out.
