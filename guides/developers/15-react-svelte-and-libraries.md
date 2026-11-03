# 15. React, Svelte and libraries

A pack's module is plain JavaScript, but what it draws with is yours: React, a charting library, your own code. This chapter is what the sandbox's frames provide and what a pack bundles, how to render React inside `mount`, where Svelte stands, `draw` for components that take their time, and library packs, which several packs share.

## What the frames provide

Every frame a host runs components in has one copy of each runtime of the spec's bindings (spec [§7.6](https://compmd.hakanserce.com/spec/#76-bindings)), shared by every component in it, through its import map:

| Runtime | The import names a module may use |
|---|---|
| React 19 | `react`, `react/jsx-runtime` |
| React DOM 19 | `react-dom`, `react-dom/client` |
| Svelte 5 | `svelte`, `svelte/internal/client`, `svelte/internal/disclose-version`, `svelte/internal/flags/legacy`, `svelte/internal/flags/async`, `svelte/reactivity`, `svelte/store`, `svelte/transition`, `svelte/easing`, `svelte/motion`, `svelte/animate`, `svelte/events`, `svelte/attachments` |

A module never bundles a runtime: two copies of React in one frame break its hooks. `compmd-pack build` leaves exactly these names as imports, and bundles everything else, a subpath not in the table too. For each runtime a module imports, it adds the runtime to `package.json`'s `peerDependencies`, at the version installed in the pack (`"react": "^19.3.0"`), and says so when it writes it. A host whose frames have an older version reports W121 on the document's import of the pack when it resolves it, before anything renders, and the pack's components that need the runtime show their fallbacks. `@compmd/sandbox`'s frames have React 19.3 and Svelte 5.57 today.

## React inside mount

A React component in a pack is rendered by its `mount`: make a root in the box, render into it on every update, unmount it on destroy. `counter` is a button that counts the reader's clicks. Its pack is the template's, with React added. Its document imports `./dist/counter.js`:

````md snippet=counter-pack/index.comp.md
---
md:
  import:
    - from: ./dist/counter.js
      names: [counter]
  export:
    pack: "@you/counter"
    version: "0.1.0"
    names: [counter]
---

# @you/counter

`counter`, a leaf in React, which counts the reader's clicks: `:: counter[Cups of tea: 0] {label="Cups of tea"}`.

```compmd component=counter signature
params:
  label: string
  start: number = 0
```
````

and the build makes it from `src/counter.tsx`:

```tsx snippet=counter-pack/src/counter.tsx
// `counter`: a button that counts the reader's clicks, drawn with React inside the pack's `mount`.
import { component } from "@compmd/pack/component";
import { useState } from "react";
import { flushSync } from "react-dom";
import { createRoot } from "react-dom/client";

/** The params, as the signature in `index.comp.md` declares them. */
export interface Params {
  label: string;
  start: number;
}

function Counter({ label, start }: Params) {
  const [count, setCount] = useState(start);
  return (
    <button type="button" onClick={() => setCount(count + 1)}>
      {label}: {count}
    </button>
  );
}

export default component<Params>((target, { params }) => {
  const root = createRoot(target);
  const render = (next: Params) => {
    flushSync(() => {
      root.render(<Counter {...next} />);
    });
  };
  render(params);
  return {
    update: render,
    destroy: () => {
      root.unmount();
    },
  };
});
```

- **The same root on every update**, so React keeps the component's state: the count stays when `label` changes.
- **`flushSync`**, so the box holds what React drew when `mount` and `update` return, as the host expects; it is how `@compmd/render`'s own React binding renders.
- **JSX** compiles with React's automatic runtime, which `@compmd/pack`'s TypeScript configuration sets (`"jsx": "react-jsx"`), so the module imports `react/jsx-runtime`, which the frames have.
- **Slots** are elements the host rendered: place each item's `element` in a node React leaves alone, such as an empty `<div>` you append it to from a `ref`, and never let React render over it.
- **The rest of the context** is yours to pass down as props: `fallback.text` for an `aria-label`, `scheme` for a palette, `set` for a bound input.

React is installed for the build and the types, as the template's other tools are, `npm install -D react react-dom @types/react @types/react-dom`:

```json snippet=counter-pack/package.json
{
  "name": "@you/counter",
  "version": "0.1.0",
  "description": "A CompMD pack: a counter the reader clicks, in React.",
  "keywords": ["compmd", "compmd-pack"],
  "license": "MIT",
  "type": "module",
  "compmd": "./index.comp.md",
  "files": ["dist", "index.comp.md", "README.md"],
  "scripts": {
    "build": "compmd-pack build",
    "dev": "compmd-pack dev",
    "test": "compmd-pack test",
    "lint": "tsc --noEmit",
    "prepublishOnly": "compmd-pack build && compmd-pack test"
  },
  "devDependencies": {
    "@compmd/pack": "0.3.0",
    "@types/react": "19.3.0",
    "@types/react-dom": "19.3.0",
    "happy-dom": "21.0.3",
    "react": "19.3.0",
    "react-dom": "19.3.0",
    "typescript": "6.0.3",
    "vitest": "5.0.5"
  }
}
```

`npm run build` leaves `react`, `react/jsx-runtime`, `react-dom` and `react-dom/client` as the module's imports, which the frames give, and bundles the rest: your code and `@compmd/pack/component`. It adds the two runtimes to `package.json`, at the versions installed, and says so:

```json
"peerDependencies": {
  "react": "^19.3.0",
  "react-dom": "^19.3.0"
}
```

Its `tsconfig.json` is the template's:

```json snippet=counter-pack/tsconfig.json
{ "extends": "@compmd/pack/tsconfig.json", "include": ["src"] }
```

Its test mounts it as a host does, clicks it, and updates its params. React draws the new count after `click()` returns, not during it, so the test waits for it with `expect.poll`:

```ts snippet=counter-pack/src/counter.test.ts
import { mountForTest } from "@compmd/pack/testing";
import { describe, expect, it } from "vitest";
import mount from "./counter.tsx";

describe("counter", () => {
  it("counts the reader's clicks from its start, and keeps the count when its label changes", async () => {
    const shown = mountForTest(mount, { params: { label: "Cups of tea", start: 2 }, fallback: "Cups of tea: 2" });
    const button = () => shown.target.querySelector("button");
    expect(button()?.textContent).toBe("Cups of tea: 2");
    button()?.click();
    await expect.poll(() => button()?.textContent).toBe("Cups of tea: 3");
    shown.update({ label: "Cups of coffee", start: 2 });
    expect(button()?.textContent).toBe("Cups of coffee: 3");
    shown.destroy();
    expect(shown.target.textContent).toBe("");
  });
});
```

A document uses it as any leaf:

```text
---
md:
  import:
    - from: "@you/counter@0"
      names: [counter]
---

:: counter[Cups of tea: 0] {label="Cups of tea"}
```

A component like this is what [chapter 10](10-your-apps-own-components.md) writes for an app; in a pack it runs in the strict sandbox, in a frame with no network, whoever imports it.

## Svelte

The frames provide Svelte 5, for the Svelte components documents define in their own fences and that apps build with themselves. `compmd-pack build` has no Svelte compiler yet: it builds TypeScript and JSX with Vite's own transform, and a `.svelte` file it imports doesn't build. So a pack's components are written in plain TypeScript or React today; a pack with Svelte components waits for the build to compile them.

## What a pack bundles

Each module the pack's document imports, `./dist/<name>.js`, is built from `src/<name>.ts` (or `.tsx`, `.js`, `.jsx`) into one ES module:

- **Everything it imports is in it**, but the runtimes above and the library packs the pack names in `peerDependencies` (below): the libraries from npm, your own files, `@compmd/pack/component`, which is a few KB. A bundler keeps only what the module uses.
- **One file, minified**, with no comments naming the files it was built from, and code splitting off: a library that loads parts of itself on demand stays in the one file, since a host that fetches the pack from a registry loads the module as text, where a second file has no path to load from. The build's check fails when a script is left beside the modules.
- **The licences of what it bundles** go into `dist/LICENSES.md`, or `dist/<name>.LICENSES.md` for each module of a pack with several. `files` publishes them with `dist/`.
- **Nothing from the network.** A strict frame has none, so fonts, icons and data the component needs are in the module, or come in its params.

A pack is at most 10 MB unpacked, the size a host takes by default, which the build checks. The Mermaid pack, the whole of Mermaid with its layout engines, is 5.2 MB built.

## Drawing asynchronously

A component that draws with a library that takes its time, a diagram or a chart that is laid out asynchronously, would write the same code each time: the newest params win, a slow draw doesn't overwrite a newer one, the drawing follows the scheme, and a draw that can't be made calls `fail`. `draw` from `@compmd/pack/component` is that code. Its pack:

````md snippet=sparkline-pack/index.comp.md
---
md:
  import:
    - from: ./dist/sparkline.js
      names: [sparkline]
  export:
    pack: "@you/sparkline"
    version: "0.1.0"
    names: [sparkline]
---

# @you/sparkline

`sparkline`, a leaf: `:: sparkline[Visits rose all week] {values=[3, 5, 4, 8, 12]}`.

```compmd component=sparkline signature
params:
  values: array
```
````

and its component, whose `render` gets the params and `{ scheme, signal }`, and returns an element, or SVG or HTML as text:

```ts snippet=sparkline-pack/src/sparkline.ts
// `sparkline`: a line through a list of numbers, drawn with `draw`, which follows the page's colour scheme, lets the
// newest params win, and fails with the message of what `render` throws.
import { draw } from "@compmd/pack/component";

/** The params, as the signature in `index.comp.md` declares them. An `array` may hold anything a document gives. */
export interface Params {
  values: unknown[];
}

const WIDTH = 120;
const HEIGHT = 32;

export default draw<Params>(
  ({ values }, { scheme }) => {
    const numbers = values.filter((value): value is number => typeof value === "number" && Number.isFinite(value));
    if (numbers.length < 2 || numbers.length !== values.length) throw new Error("a sparkline needs two or more numbers, and nothing else");
    const low = Math.min(...numbers);
    const span = Math.max(...numbers) - low || 1;
    const points = numbers.map((n, i) => `${String((i * WIDTH) / (numbers.length - 1))},${String(HEIGHT - ((n - low) * HEIGHT) / span)}`);
    // Only numbers go into the markup, so a document can't write any of its own into it.
    const stroke = scheme === "dark" ? "#7ee2b8" : "#0b7a52";
    return `<svg viewBox="-2 -2 ${String(WIDTH + 4)} ${String(HEIGHT + 4)}" width="${String(WIDTH)}" height="${String(HEIGHT)}"><polyline fill="none" stroke="${stroke}" stroke-width="2" points="${points.join(" ")}"/></svg>`;
  },
  { role: "img" },
);
```

What `draw` does with it:

- **`render` may return a promise.** The drawing replaces the last one only once it is ready, so the box never flickers empty. When an update overtakes a draw, the older one is dropped and its `signal` aborted: pass the signal on to a library that takes one, or check `signal.aborted` between steps of a long draw.
- **A throw, or a rejected promise, fails** with its message: the host reports W118 and shows the fallback until the next update, which draws again. Throw for what the params can't draw, as `sparkline` does for a list with a word in it.
- **The scheme**: a change comes as an update with the same params, so `render` runs again with the new `scheme`.
- **`role`** gives the drawing a role, with `fallback.text` as its `aria-label`; `img` for a picture.
- **`oneAtATime: true`** runs one draw at a time in a frame, for a library with one global configuration, as Mermaid has, so two components' themes never mix.
- **Text is set as HTML.** A string `render` returns becomes the drawing's markup, so never put a document's text in it unescaped: `sparkline` writes only numbers it checked, and a library that renders markup should sanitize it, as Mermaid does with `securityLevel: "strict"`.

`destroy` aborts the draw in progress and empties the box. A document uses it as any leaf:

```text
---
md:
  import:
    - from: "@you/sparkline@0"
      names: [sparkline]
---

:: sparkline[Visits rose from 3 to 12] {values=[3, 5, 4, 8, 12]}
```

Its tests wait for the drawing with `settled()`:

```ts snippet=sparkline-pack/src/sparkline.test.ts
import { mountForTest } from "@compmd/pack/testing";
import { describe, expect, it } from "vitest";
import mount from "./sparkline.ts";

describe("sparkline", () => {
  it("draws a line labelled with its fallback, and again in the other scheme", async () => {
    const shown = mountForTest(mount, { params: { values: [3, 5, 4, 8] }, fallback: "Visits rose all week" });
    await shown.settled();
    const image = shown.target.querySelector("[role=img]");
    expect(image?.getAttribute("aria-label")).toBe("Visits rose all week");
    expect(image?.querySelector("polyline")?.getAttribute("stroke")).toBe("#0b7a52");
    shown.setScheme("dark");
    await shown.settled();
    expect(shown.target.querySelector("polyline")?.getAttribute("stroke")).toBe("#7ee2b8");
    shown.destroy();
  });

  it("fails with what it can't draw, shows the fallback, and draws again at the next update", async () => {
    const shown = mountForTest(mount, { params: { values: [3, "lots"] }, fallback: "Visits rose all week" });
    await shown.settled();
    expect(shown.failures).toEqual(["a sparkline needs two or more numbers, and nothing else"]);
    expect(shown.target.textContent).toBe("Visits rose all week");
    shown.update({ values: [3, 5] });
    await shown.settled();
    expect(shown.target.querySelector("polyline")).not.toBeNull();
    shown.destroy();
  });
});
```

Its `tsconfig.json` is the template's:

```json snippet=sparkline-pack/tsconfig.json
{ "extends": "@compmd/pack/tsconfig.json", "include": ["src"] }
```

and its `package.json` the template's, with its own name and description:

```json snippet=sparkline-pack/package.json
{
  "name": "@you/sparkline",
  "version": "0.1.0",
  "description": "A CompMD pack: a sparkline of a list of numbers, in the page's colour scheme.",
  "keywords": ["compmd", "compmd-pack"],
  "license": "MIT",
  "type": "module",
  "compmd": "./index.comp.md",
  "files": ["dist", "index.comp.md", "README.md"],
  "scripts": {
    "build": "compmd-pack build",
    "dev": "compmd-pack dev",
    "test": "compmd-pack test",
    "lint": "tsc --noEmit",
    "prepublishOnly": "compmd-pack build && compmd-pack test"
  },
  "devDependencies": {
    "@compmd/pack": "0.3.0",
    "happy-dom": "21.0.3",
    "typescript": "6.0.3",
    "vitest": "5.0.5"
  }
}
```

## What component code may import

Whatever a pack bundles, its module's imports, and those of every component a host runs, resolve to four kinds of module only (spec [§7.6](https://compmd.hakanserce.com/spec/#76-bindings)):

| An import of | Resolves to | Where it can't |
|---|---|---|
| a runtime, `react`, `svelte`, … | the frame's one copy | W121 on the import, at resolve, when the pack's `peerDependencies` ask for a version the host doesn't have |
| a library the host offers, by its npm name (`d3`) | the host's copy, listed under `libraries` in its configuration | the component shows its fallback with W121 on a host that doesn't offer it |
| a library pack the pack names in `peerDependencies` | one copy for every pack in the document | W121 on the import, at resolve |
| the module's own files, by relative path | files in its pack | a host that loads the module as text can't load them: W121 |

Anything else resolves nowhere, and the component shows its fallback with W121. A library the host offers isn't portable, as an ambient pack's names aren't: bundle it, or depend on a library pack, so the pack works everywhere.

## Library packs

A pack bundles every library it uses, which is right for most. A large library several packs use whole, such as three.js or KaTeX, can instead be a **library pack**: the library built as one module, which every pack in a document shares, so a frame loads one copy. Anyone may publish one.

```sh
npm create @compmd/pack three -- --name @kit/three --library three
cd three
npm install
npm run build
```

It writes a pack with no document, no component, no tests and no test page:

```text
three/
  package.json     "compmd": { "library": "dist/index.js" }, "exports" naming the same file, three in devDependencies
  src/index.ts     export * from "three";
  tsconfig.json    @compmd/pack's
  README.md        how a pack uses it, when to add "single", how to publish
  .gitignore
```

- **`src/index.ts` re-exports the library**: all of it, or only what packs use, to make the module smaller. A library with a default export needs `export { default } from "three"` as well. It must import one npm package, the library, or the build says so.
- **Its version is the library's.** `@kit/three@0.170.0` is `three@0.170.0`: pin `three` in `devDependencies` at the version to publish, and `npm run build` writes the installed version into `package.json`, with a warning; its check fails when they differ. Its `prepublishOnly` is the build.
- **`"single": true`**, beside `library` in the `compmd` field, says the library can't have two copies in one frame: it defines custom elements, patches globals, or keeps one instance by design. A pack whose range doesn't match the version the document already has then gets W121 on its import instead of a second copy, and its components that need it show their fallbacks.

A pack that uses one:

1. `npm install -D @kit/three`, for the build and the types.
2. Name it in `peerDependencies`, with the range the components work with: `"@kit/three": "^0.170"`. The build leaves every name there, and its paths, out of the module.
3. Import it by that name: `import * as THREE from "@kit/three"`.
4. For the test page, list it in a `compmd.config.json` beside `demo.comp.md`: `{ "libraries": ["@kit/three"] }`.

A host that fetches packs from a registry fetches the library pack the same way, and gives every pack in the document the newest version all their ranges allow. An app that installs its packs instead installs the library pack too, and lists it under `libraries` in its configuration, as the test page does: `@compmd/vite` serves an installed pack's modules by URL, and a frame shares a library pack only with a module it was given as code, as a pack fetched from a registry is. A peer no host can give is W121 on the import, with why.

Next: [code renderers](16-code-renderers.md), components that draw a language's code blocks.
