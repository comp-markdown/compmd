# __pack__

A [CompMD](https://compmd.hakanserce.com/) pack with one component, `__component__`, as one built module that a host loads by URL and runs in its sandbox.

## Use it in a document

```md
---
md:
  import:
    - from: "__pack__@0"
      names: [__component__]
---
```

## Make it yours

- `src/__component__.ts`: the component, a `mount(target, context)` function in the plain binding. It draws into `target`, and returns `update` for new params and `destroy`. To write it in React or Svelte, render your framework inside `mount`: the sandbox provides React 19 and Svelte 5, so they stay out of the bundle.
- `index.comp.md`: the pack's document, with the component's signature: its params, their types and defaults. A document's params are checked against it, so a reader sees a mistake before anything runs.
- `demo/demo.comp.md`: the test page's document.

Everything else the component imports is bundled into `dist/__component__.js`, so the pack works offline, with no network in the sandbox. It stays one file, with no chunks, so a host that fetches the pack from a registry can load it.

## Share a library

A large library that several packs use (three.js, KaTeX) can be shared instead of bundled, through a library pack, so a sandbox loads one copy (`npm create @compmd/pack three -- --library three` makes one):

1. `npm install -D @kit/three`, the library pack.
2. Add it to `package.json`'s `peerDependencies`, with the range the component works with: `"@kit/three": "^0.170"`. The build leaves every name there out of the bundle.
3. Import it by that name in `src/__component__.ts`: `import * as THREE from "@kit/three"`.
4. List it in `demo/compmd.config.json`'s `libraries` (`"libraries": ["@kit/three"]`), so the test page offers it.

A host that fetches the pack from a registry gets the library from the registry too, one version for every pack in a document.

## Try it

```sh
npm install
npm run dev     # the test page, in the strict sandbox; it reloads as you change src/
npm test        # the component in happy-dom
```

## Publish it

1. Change the version in both `package.json` and `index.comp.md`'s `export.version`, the same in each; `npm publish` checks they match.
2. `npm login`, then `npm publish --access public`. It builds and tests first, and publishes `dist/`, `index.comp.md` and this README.
3. For provenance, publish from CI with npm's trusted publishing instead (`npm publish --provenance`, from a GitHub Actions workflow the package's settings on npmjs.com name).

A host adds the pack with `npm install __pack__`; with `@compmd/vite`, nothing else is needed for documents to import it. A host with registries in its configuration fetches it from the registry when a document imports it.
