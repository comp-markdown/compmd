# 16. Testing and publishing

A pack is ready to publish when its components are tested, its build has passed the checks a host would make, and its `package.json` says what it is. This chapter takes a small pack, a progress bar, from its tests to npm, and then to an app that uses it.

## The pack

`npm create @compmd/pack` writes a pack like this one ([chapter 12](README.md#contents)): a component in TypeScript, its signature in the pack's document, its tests, and a `package.json` whose scripts run `@compmd/pack`'s command, which builds, checks, tests and serves the pack, so the pack holds only what is its own.

```ts snippet=progress-pack/src/progress.ts
import { component } from "@compmd/pack/component";

/** The params, as the signature in `index.comp.md` declares them. */
export interface Params {
  value: number;
  max: number;
}

export default component<Params>((target, { params, fallback }) => {
  const bar = document.createElement("progress");
  // A screen reader hears the fallback, which says in words what the bar shows.
  bar.setAttribute("aria-label", fallback.text);
  const numbers = document.createElement("span");
  target.append(bar, " ", numbers);
  const draw = ({ value, max }: Params) => {
    bar.max = max;
    bar.value = Math.min(Math.max(value, 0), max);
    numbers.textContent = `${String(value)} of ${String(max)}`;
  };
  draw(params);
  return {
    update: draw,
    destroy: () => {
      bar.remove();
      numbers.remove();
    },
  };
});
```

The pack's document imports the component's built module, exports it under the pack's name and version, and declares its signature, which hosts check every use against before anything runs:

````md snippet=progress-pack/index.comp.md
---
md:
  import:
    - from: ./dist/progress.js
      names: [progress]
  export:
    pack: "@you/progress"
    version: "0.1.0"
    names: [progress]
---

# @you/progress

`progress`, a leaf: `:: progress[3 of 4 done] {value=3 max=4}`.

```compmd component=progress signature
params:
  value: number
  max: number = 100
```
````

Its `tsconfig.json` is one line, `@compmd/pack`'s:

```json snippet=progress-pack/tsconfig.json
{ "extends": "@compmd/pack/tsconfig.json", "include": ["src"] }
```

## Testing a component

`@compmd/pack/testing`'s `mountForTest` runs a component as a host does, through `@compmd/render`'s component layer, with the params, the fallback and the colour scheme you give it:

```ts snippet=progress-pack/src/progress.test.ts
import { mountForTest } from "@compmd/pack/testing";
import { describe, expect, it } from "vitest";
import mount from "./progress.ts";

describe("progress", () => {
  it("draws its value of its max, with its fallback as its label", () => {
    const shown = mountForTest(mount, { params: { value: 3, max: 4 }, fallback: "3 of 4 done" });
    const bar = shown.target.querySelector("progress");
    expect(bar?.value).toBe(3);
    expect(bar?.max).toBe(4);
    expect(bar?.getAttribute("aria-label")).toBe("3 of 4 done");
    expect(shown.target.textContent).toContain("3 of 4");
  });

  it("follows new params, keeps a value outside its range inside the bar, and goes when destroyed", () => {
    const shown = mountForTest(mount, { params: { value: 1, max: 10 }, fallback: "1 of 10" });
    shown.update({ value: 12, max: 10 });
    expect(shown.target.querySelector("progress")?.value).toBe(10);
    expect(shown.target.textContent).toContain("12 of 10");
    shown.destroy();
    expect(shown.target.querySelector("progress")).toBeNull();
  });
});
```

`npm test` runs `compmd-pack test`: every `src/**/*.test.ts` in happy-dom, with no test configuration in the pack. A test has the handle's `target`, what the component `set` (`sets`, for the params you name in `inputs`), what it failed with (`failures`), `setScheme` for an update in the other colour scheme, and, for a component that draws asynchronously, `settled()`, which waits for its drawing.

Params come from documents, so test what a document may give: here a value past its max stays inside the bar. A signature checks types and the values an enum allows, never that two params agree.

`npm run dev` serves the test page: a `demo.comp.md` beside the pack's document, which the template writes, rendered in the strict sandbox as a host renders it, light and dark. It shows what the tests can't see, such as how the bar looks.

## What the build checks

`npm run build` runs `compmd-pack build`: each component's module, built from `src/` into `dist/` as one minified ES module with every library bundled in but the runtimes the frames give (React, React DOM and Svelte, which it adds to `peerDependencies` at the version it was built with). Then it checks the pack as a host would, and fails, saying what to fix, when:

- the document's `export.version` isn't `package.json`'s version: a host reports `F022`;
- a file the document imports isn't there, or isn't in `package.json`'s `files`, so it wouldn't be published;
- a module isn't one file: a host that loads it as text can't load its chunks;
- the pack is over 10 MB unpacked, the size a host takes by default;
- a library pack's version isn't its library's.

## A pack's `package.json`

```json snippet=progress-pack/package.json
{
  "name": "@you/progress",
  "version": "0.1.0",
  "description": "A CompMD pack: a progress bar with its numbers.",
  "keywords": ["compmd", "compmd-pack"],
  "license": "MIT",
  "type": "module",
  "compmd": "./index.comp.md",
  "files": ["dist", "index.comp.md", "README.md"],
  "scripts": {
    "build": "compmd-pack build",
    "dev": "compmd-pack dev",
    "test": "compmd-pack test",
    "prepublishOnly": "compmd-pack build && compmd-pack test"
  },
  "devDependencies": {
    "@compmd/pack": "^0.3.0"
  }
}
```

What makes it a pack on a registry ([*CompMD packs on a registry*](https://compmd.hakanserce.com/spec/#registry), section 2):

- **A scoped name**, `@you/progress`, the name documents import. No prefix is needed; a scope with a library of the same name may call its pack `@you/compmd-progress`.
- **The `compmd` field**, naming the pack's document, even when it is the default `index.comp.md`: a registry keeps every field, so a tool can tell a pack from other packages without downloading it.
- **The keyword `compmd-pack`**, which only packs use, so a search for `keywords:compmd-pack` finds them. Tools that help build packs use `compmd`.
- **`files`**: what a host reads, the built modules, the document and the files it imports, and the README for npm's page; never the sources or the tests, so the download stays small.
- **`dependencies`** are never installed by a host: what the components use is in their modules, and the packs the pack imports are in its document's `md.import`, which the host resolves.

## Publishing

1. Raise the version in `package.json` and in the document's `export.version`, the same in both.
2. `npm login`, then `npm publish --access public`. `prepublishOnly` builds, checks and tests first, so a pack that fails never goes out.

That's a pack on npm. For **provenance**, a record of where and from what it was built, which npm shows on the pack's page, publish from CI with npm's trusted publishing instead: no token is stored anywhere, GitHub proves to npm which repository and workflow publishes. On npmjs.com, the pack's settings name the workflow, under **Trusted publisher**; the workflow needs `id-token: write`, and npm 11.5.1 or later:

```yaml
name: Release
on:
  push:
    tags: ["v*"]
permissions:
  contents: read
  id-token: write
jobs:
  release:
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@v7
      - uses: actions/setup-node@v7
        with:
          node-version: 24
      - run: npm install --global npm@11
      - run: npm ci
      - run: npm publish --access public
```

npm adds the provenance when it publishes with a trusted publisher. A package must exist before a trusted publisher can be set up for it, so publish its first version by hand.

Each version's tarball has a hash, its integrity, which npm publishes (`npm view @you/progress@0.1.0 dist.integrity`). A document that pins the version writes it as its import's `integrity`, and a host refuses a pack that doesn't match.

## Sharing a large library

A pack bundles every library its components use, which is right for most: a bundler keeps only what the pack imports. A large library several packs use whole, such as three.js or KaTeX, can instead be shared, so a sandbox loads one copy for every pack in a document:

- **A library pack** holds the library, built as one module: `npm create @compmd/pack three -- --name @kit/three --library three` writes one, with `"compmd": { "library": "dist/index.js" }` and the library's version as its own. Anyone may publish one, the library's own authors included.
- **A pack that uses it** names it in `peerDependencies` (`"@kit/three": "^0.170"`) and imports it by that name; the build leaves it out of the module.
- **A host** that doesn't offer a library of that name fetches it from its registries like a pack, and gives every pack in a document the newest version all their ranges allow. A library that can't have two copies in one sandbox says so, with `"single": true`; a pack asking for a version the document can't have then gets `W121`, and its components that don't need the library still render.

## A host adding the pack

An app that wants the pack installs it, `npm install @you/progress`, and its documents import it, `from: "@you/progress@0"`, then write `:: progress[3 of 4 done] {value=3 max=4}`: with `@compmd/vite`, nothing else ([chapter 9](09-packs-from-npm.md)). An app with registries in its configuration doesn't even install it: it fetches the pack when a document imports it. Either way it runs strict, unless the app's configuration lists it for more.

Next: Part V, beyond Vite *(to come)*.
