<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/comp-markdown/compmd/main/branding/compmd-dark.svg">
  <img src="https://raw.githubusercontent.com/comp-markdown/compmd/main/branding/compmd-light.svg" alt="CompMD" width="200">
</picture>

# @compmd/pack

Everything a [CompMD](https://compmd.dev/) pack would otherwise copy into itself: `compmd-pack` builds a pack, checks it before it is published, runs its tests and serves its test page, and two small modules help write and test its components. A pack holds only what is its own: its components, their signatures, a document to try them in, and their tests. Upgrading this package updates the build, the checks and the test page of every pack. The design is [writing a pack](https://github.com/comp-markdown/compmd/blob/main/docs/writing-a-pack.md).

**Website:** [compmd.dev](https://compmd.dev/), with the spec and a playground to try CompMD in.

`npm create @compmd/pack` writes a pack on it ([`@compmd/create-pack`](https://github.com/comp-markdown/compmd/blob/main/npm-packages/create-pack/README.md)). It shares its version with the other `@compmd` packages.

**Versions:**
- **0.11.0:** the build's check covers a pack's data files and its definitions' images and links, reads `files` as npm does (the whole folder without it), and refuses a path that leaves the pack. `compmd-pack dev` queues rebuilds one after another, logs their errors instead of stopping, and follows added and removed files. On the sandbox's 0.11.0. Breaking, for packs: one that names a data file, an image or a link it doesn't publish fails its build.
- **0.10.0:** a component fits its box: `fitBox` and `fillBox` in `@compmd/pack/component`; `draw()` fits a drawing that is one SVG, `img` or `canvas` by the use's `object-fit` and `object-position`, draws again when they or the box's type change, with `fit` and `type` in its context, and keeps the styles of a drawing given as markup in a page that refuses inline styles; `mountForTest`'s `fit` and `setFit`; the test page lays each use out as in the page, with **Show boxes**. A pack of an npm package, which `npm create @compmd/pack -- --from` writes: its build writes the signature between the `compmd-pack:generated` comments from the package's types and says what changed, compiles a Svelte package's sources, and checks each module for what a strict frame refuses; `@compmd/pack/wrap` has the parts, for tools. The build checks that each font file the document declares is there and published, and warns of one with no licence beside it. On the sandbox's 0.10.0.
- **0.9.0:** the same as 0.8.0, on the sandbox's 0.9.0.
- **0.8.0:** the same as 0.7.1, on the sandbox's 0.8.0.
- **0.7.1:** the same as 0.7.0; its homepage and links name the website's new address, [compmd.dev](https://compmd.dev/).
- **0.7.0:** the same as 0.6.0, on the sandbox's 0.7.0.
- **0.6.0:** the same as 0.5.0, on the sandbox's 0.6.0.
- **0.5.0:** `draw` tells the page it is drawing (`context.busy`, spec §7.4) until the drawing shows or a newer one overtakes it, on a renderer that has `busy`; `mountForTest`'s `settled()` waits for what a component gave `busy` too.
- **0.4.0:** built modules minified in full, without the comments naming the files they were built from, and built for production whatever `NODE_ENV` says, so a TSX component built under a test runner loads in the frames.
- **0.2.0:** the first version.

## The command

A pack's scripts run it in the pack's folder:

```json
"scripts": {
  "build": "compmd-pack build",
  "dev": "compmd-pack dev",
  "test": "compmd-pack test",
  "prepublishOnly": "compmd-pack build && compmd-pack test"
}
```

| Command | What it does |
|---|---|
| `compmd-pack build` | Builds each module the pack publishes, then checks the pack as a host would (below). |
| `compmd-pack dev [--port <n>]` | Serves the test page: `demo.comp.md`, rendered in host mode with the pack as a folder pack, each use in a strict frame of its own, laid out as in the page, its diagnostics and failures under it, a switch between the reader's colour scheme, light and dark, and **Show boxes**, which outlines each use's box. Saving a source builds its module again, and the page reloads. Nothing of the page is written into the pack. |
| `compmd-pack test [filter…]` | Runs `src/**/*.test.ts` in happy-dom, with no Vitest configuration in the pack. |

**What it builds.** The pack's document (`package.json`'s `compmd`) imports each component's built module, `./dist/<name>.js`, which is built from `src/<name>.ts` (or `.tsx`, `.js`, `.jsx`): one ES module, minified, with every library bundled in, but the runtimes the sandbox's frames provide (React, React DOM, Svelte; spec §7.6) and the library packs the pack names in `peerDependencies`, which packs share. Code splitting is off, so a library that loads parts of itself on demand stays in the one file, which a host that loads the module as text needs. The build adds each runtime a module imports to `peerDependencies`, at the version it was built with, and writes the licences of what it bundles to `dist/LICENSES.md`. A library pack (`"compmd": { "library": "dist/index.js" }`) is built from `src/index.ts`, which re-exports its library, and takes the library's version as its own.

**What it checks.** That the document's `export.version` is `package.json`'s (F022 otherwise); that every file its documents name is there, and published by `files` (read as npm reads it, globs and folders, or the whole folder without it): what they import, their data files, their fonts' files, and the images and links in their definitions; that none of them leaves the pack; that each module is one file, with no chunk beside it; that the pack is under the 10 MB a host takes by default; and that a library pack's version is its library's. Each problem says what to fix, and the build fails.

**A pack of an npm package.** A pack `npm create @compmd/pack -- --from <package>` wrote has a `compmd.wrap.json` ([wrapping an npm package](https://github.com/comp-markdown/compmd/blob/main/docs/wrapping-an-npm-package.md)). Its build writes the signature fence between the document's `compmd-pack:generated` comments first, from the package's TypeScript types (its own props only, never React's or the DOM's), kept to the props `params` lists, with `overrides` over them, and says what changed: a param added, one gone, one whose type changed, and a prop the package has that isn't a param. A param that went fails the build while the demo or the overrides still name it. Without the comments the fence is the author's: the build leaves it, and says where the types differ. A Svelte package's sources are compiled with the Svelte the frames provide, their CSS injected by the module. Then each module is checked for what a strict frame refuses: a URL of another origin, a worker, code made from text, WebGL. `@compmd/pack/wrap` has the parts, for tools.

**A library pack in the test page.** A pack that shares a library pack lists it in a `compmd.config.json` beside `demo.comp.md` (`{ "libraries": ["@kit/three"] }`): the test page reads it, with the pack added as a folder pack.

## Writing a component

`@compmd/pack/component` is bundled into the pack's module; it is a few KB.

```ts
import { component } from "@compmd/pack/component";

export default component<{ value: string }>((target, { params }) => {
  target.textContent = params.value;
  return { update: (next) => { target.textContent = next.value; } };
});
```

`component<Params>` types a plain component's params as its signature declares them. `context.scheme` is the colour scheme the page shows it in, and a change of scheme comes as an update; `context.fail(message)` is for a drawing that can't be made, found after `mount` returned (spec §7.4).

A component fits its box, and has no size params of its own. `fitBox(drawing, context.fit)` gives a drawing with an aspect ratio of its own (an SVG, an `img`, a `canvas`) the box's size, scaled and placed as the use's `object-fit` and `object-position` say. `fillBox(element)` is for a drawing with no ratio, sized in script from its box (a chart, a map): it fills the box, and what it drew last never counts as its size.

```ts
import { component, fitBox } from "@compmd/pack/component";

export default component<{ value: string }>((target, context) => {
  const code = target.appendChild(qrCode(context.params.value));   // an SVG with a viewBox
  fitBox(code, context.fit);
  return { update: () => fitBox(code, context.fit) };
});
```

A component that draws asynchronously, with a library that renders a diagram, uses `draw`:

```ts
import mermaid from "mermaid";
import { draw } from "@compmd/pack/component";

let drawn = 0;
export default draw<{ code: string }>(
  async ({ code }, { scheme }) => {
    mermaid.initialize({ startOnLoad: false, securityLevel: "strict", theme: scheme === "dark" ? "dark" : "default" });
    return (await mermaid.render(`mermaid-${String((drawn += 1))}`, code)).svg;
  },
  { role: "img", oneAtATime: true },
);
```

`render(params, { scheme, fit, signal })` returns an element, or SVG or HTML as text. A drawing that is one SVG, `img` or `canvas` is fitted in the component's box, which the document sizes (`width`, `height`, `aspect-ratio`), as its `object-fit` and `object-position` say. The newest params win: a draw a newer one overtakes is dropped (its `signal` aborted), and the last drawing stays until the next is ready. A draw that throws calls `context.fail` with its message, so the host reports W118 and shows the fallback until the next update. `role` gives the drawing a role, with the fallback's text as its label; `oneAtATime` runs one draw at a time per frame, for a library with one global configuration.

## Testing a component

```ts
import { mountForTest } from "@compmd/pack/testing";
import mount from "./mermaid.ts";

const chart = mountForTest(mount, { params: { code: "graph LR; A --> B" }, fallback: "A to B", scheme: "dark" });
await chart.settled();          // its draws are done
chart.update({ code: "graph LR; A -->" });
await chart.settled();
chart.failures;                 // ["Parse error on line 1: …"]
chart.target.textContent;       // "A to B": the fallback, until the next update
chart.setScheme("light");       // an update with the same params
chart.setFit({ mode: "cover", x: 0.5, y: 0.5 });   // as `object-fit=cover` on the use
chart.destroy();
```

`mountForTest` runs the component through `@compmd/render`'s component layer, as a host does, in the DOM `compmd-pack test` gives: `failures` are what it failed with, `sets` what it `set` (for the params named in `inputs`).

## Commands

```sh
pnpm install         # in the repository root
pnpm build           # the command and the modules into dist/
pnpm test            # reading, building and checking packs in Node; the helpers and the kit in happy-dom
pnpm lint            # eslint and the type check
```

The test page and the command run end to end in [`@compmd/create-pack`'s tests](https://github.com/comp-markdown/compmd/blob/main/npm-packages/create-pack/test/pack.test.ts), on packs it writes, in Chrome and WebKit.

## License

MIT.
