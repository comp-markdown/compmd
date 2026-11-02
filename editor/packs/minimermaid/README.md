# @preview/minimermaid

A [CompMD](https://compmd.hakanserce.com/) pack with one component, `minimermaid`, as one built module that a host loads by URL and runs in its sandbox.

## Use it in a document

```md
---
md:
  import:
    - from: "@preview/minimermaid@0"
      names: [minimermaid]
---
```

## Make it yours

- `src/minimermaid.ts`: the component, a `mount(target, context)` function in the plain binding, written with `component` from `@compmd/pack/component` so its params are typed. It draws into `target`, and returns `update` for new params and `destroy`. To write it in React or Svelte, render your framework inside `mount`: the sandbox provides React 19 and Svelte 5, so they stay out of the bundle.
- `src/minimermaid.test.ts`: its tests, with `mountForTest` from `@compmd/pack/testing`.
- `index.comp.md`: the pack's document, with the component's signature: its params, their types and defaults. A document's params are checked against it, so a reader sees a mistake before anything runs.
- `demo.comp.md`: the test page's document.

Everything else the component imports is bundled into `dist/minimermaid.js`, so the pack works offline, with no network in the sandbox. The build, the checks before publishing and the test page are [`@compmd/pack`](../../../pack/README.md)'s.

## Try it

```sh
npm install
npm run dev     # the test page, in host mode, with a light and dark switch; it reloads as you change src/
npm test        # the component in happy-dom
npm run build   # the module, then the checks before publishing
```

## Publish it

1. Change the version in both `package.json` and `index.comp.md`'s `export.version`, the same in each; the build checks they match.
2. `npm login`, then `npm publish --access public`. It builds, checks and tests first, and publishes `dist/`, `index.comp.md` and this README.
3. For provenance, publish from CI with npm's trusted publishing instead (`npm publish --provenance`, from a GitHub Actions workflow the package's settings on npmjs.com name).

A host adds the pack with `npm install @preview/minimermaid`; with `@compmd/vite`, nothing else is needed for documents to import it.
