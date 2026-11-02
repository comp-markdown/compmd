# __pack__

A [CompMD](https://compmd.hakanserce.com/) library pack: `__library__`, built as one module that packs share, so a sandbox loads one copy of it instead of each pack bundling its own.

## Use it in a pack

1. `npm install -D __pack__`.
2. Add it to the pack's `peerDependencies`, with the range the pack works with: `"__pack__": "^1.2"`. The pack's build leaves it out of the bundle.
3. Import it by that name in the pack's components: `import { … } from "__pack__"`.

A host gives every pack in a document one version: the newest that every pack's range matches.

## Make it yours

- `src/index.ts` re-exports `__library__`. Re-export only what packs use if that makes the module smaller; the library's own default export, if it has one, needs `export { default } from "__library__"`.
- Pin `__library__` in `devDependencies` at the version to publish: this pack's version is that library's (`__pack__@1.2.0` is `__library__@1.2.0`), and `npm run build` writes it into `package.json`.
- If the library can't have two copies in one sandbox (it defines custom elements, patches globals, or keeps one instance by design), add `"single": true` to the `compmd` field: `"compmd": { "library": "dist/index.js", "single": true }`. A pack that asks for another version then gets a warning instead of a second copy.

Everything `__library__` imports is bundled into `dist/index.js`, but React, React DOM and Svelte, which the sandbox provides (the build adds the ones it imports to `peerDependencies`), and other library packs named in `peerDependencies`. The build is `@compmd/pack`'s, so upgrading that package updates it; nothing of it is copied here.

## Publish it

1. `npm run build`, which writes the library's version into `package.json`, then checks the pack as a host would: the version is the library's, and the module is one file.
2. `npm login`, then `npm publish --access public`. It builds and checks first, and publishes `dist/` and this README.
3. For provenance, publish from CI with npm's trusted publishing instead (`npm publish --provenance`, from a GitHub Actions workflow the package's settings on npmjs.com name).
