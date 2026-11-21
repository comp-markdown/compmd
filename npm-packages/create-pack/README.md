<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/comp-markdown/compmd/main/branding/compmd-dark.svg">
  <img src="https://raw.githubusercontent.com/comp-markdown/compmd/main/branding/compmd-light.svg" alt="CompMD" width="200">
</picture>

# @compmd/create-pack

Writes a [CompMD](https://compmd.dev/) pack: a component as one built ES module, with every library it uses bundled in, its signature, a test page that renders it in the strict sandbox as a host does, and the steps to publish it. The design is [docs/create-pack.md](https://github.com/comp-markdown/compmd/blob/main/docs/create-pack.md).

**Website:** [compmd.dev](https://compmd.dev/), with the spec and a playground to try CompMD in.

```sh
npm create @compmd/pack mermaid -- --name @kit/mermaid --renders mermaid
cd mermaid
npm install
npm run dev
```

It asks nothing: everything is a flag, so it scripts the same way it runs.

| Flag | What it does |
|---|---|
| `--name <@scope/pack>` | the npm package; by default `@<your npm user>/<folder>`, or `@me/<folder>` when npm doesn't say who you are |
| `--component <name>` | the component's name; by default the folder's |
| `--renders <language>` | makes the component a code renderer of that language's code blocks (spec §7.7) |
| `--library <package>` | makes a library pack of that npm package instead, which packs share (below) |
| `--from <package>[@<version>]` | makes a pack of that npm package's component, installed and pinned (below) |
| `--export <name>` | with `--from`: the export that is the component (`BarChart`), or a web component's tag; by default the default export, or the only custom element |
| `--kind <kind>` | with `--from`: `react`, `svelte`, `web-component` or `library`, over what is read from the package |

It shares its version with the other `@compmd` packages, and the pack it writes pins [`@compmd/pack`](https://github.com/comp-markdown/compmd/blob/main/npm-packages/pack/README.md) and Vitest at the versions released with it, so a pack made today builds the same later.

## What it writes

Only what is the pack's own. The build, the checks before publishing and the test page are `@compmd/pack`'s, run by its scripts (`compmd-pack build`, `dev`, `test`), so upgrading that package updates them in packs made earlier ([writing a pack](https://github.com/comp-markdown/compmd/blob/main/docs/writing-a-pack.md)).

```
mermaid/
  package.json         "compmd": "./index.comp.md", the keyword compmd-pack, scripts that run compmd-pack; publishes dist/, index.comp.md and the README
  index.comp.md        the export, the import of ./dist/mermaid.js, the signature
  src/mermaid.ts       export default component<Params>((target, { params, fallback, scheme }) => { … })
  src/mermaid.test.ts  the component through the host contract, with mountForTest
  demo.comp.md         the test page's document, which imports the pack
  tsconfig.json        one line: @compmd/pack's
  README.md            what to change, how to try it, how to publish
```

The component is plain TypeScript in the plain binding (spec §7.6). React and Svelte stay out of the bundle, since the sandbox's frames provide them, so a component written in either renders it inside `mount`; the build adds each one it leaves out to `peerDependencies`, at the version it was built with, so a host with an older one reports it when it resolves the pack (spec §6.4). The module is one file, with code splitting off, so a host that fetches the pack from a registry can load it as text.

A large library that several packs use can be shared instead of bundled: the pack names a library pack in `peerDependencies`, the build leaves it out, and a host gives every pack in a document one copy. The written README gives the steps.

## Library packs

```sh
npm create @compmd/pack three -- --name @kit/three --library three
cd three
npm install
npm run build
```

writes a library pack ([CompMD packs on a registry](https://compmd.dev/spec/#registry), section 5): `src/index.ts` re-exports `three`, built as one module, `dist/index.js`, which `package.json` names in `"compmd": { "library": "dist/index.js" }`. It has no document, no component and no test page. Its version is the library's: the build writes the installed `three`'s version into `package.json`, and publishing checks it. Its README says when to add `"single": true`.

## Packs of an npm package

```sh
npm create @compmd/pack qr-code -- --name @kit/qr-code --from react-qr-code
cd qr-code
npm run dev
```

writes a pack of the package's component ([wrapping an npm package](https://github.com/comp-markdown/compmd/blob/main/docs/wrapping-an-npm-package.md)), and installs it. It reads what kind the package is: React (`react` among its peer dependencies), Svelte (`svelte` there, or a `svelte` export condition), a web component (a custom element in its types, or a `custom-elements.json`), or a plain library. Then it writes:

- `package.json` with the package at the exact version it installed: one version of the pack is one build of the package.
- `src/qr-code.ts`, the glue that mounts the component in its box with the params as its props, written once and yours to change. A plain library's is a `draw()` skeleton that imports it, for you to finish.
- `compmd.wrap.json`, the wrap's settings: the package, its export and kind, the props that are params (every one it could map, an allowlist from then on), and what you say over the types (`overrides`, `input`, `label`, `slots`, `box`).
- `index.comp.md` with the signature between `compmd-pack:generated` comments, which `compmd-pack build` writes again from the package's types at each build, saying what changed.
- `demo.comp.md`, with a use as it is and sized ones.

It lists the props it left out, and why (a callback, a size in pixels, a prop the box takes), and what in the package a strict frame would refuse (a URL of another origin, a worker, code made from text, WebGL).

## Commands

```sh
pnpm install         # in the repository root
pnpm build           # the command into dist/
pnpm test            # writing packs, the template's components, written packs built and opened in Chrome and WebKit (react-qr-code wrapped among them), and a library pack built
pnpm lint            # eslint and the type check, the template's files too
```

The template is in [`template/`](https://github.com/comp-markdown/compmd/tree/main/npm-packages/create-pack/template): the files every pack has, those of a leaf or a code renderer, and a library pack's own. Its names are `__pack__`, `__component__`, `__language__` and `__library__`, which are valid in code, so its files are linted, type-checked and tested in place (the components in happy-dom, as `compmd-pack test` runs them), but a library pack's `src/index.ts` and a wrapped pack's glue (`template/wrap/`), which import the package only a written pack has: the test that wraps `react-qr-code` checks the glue there.

## License

MIT.
