<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/comp-markdown/compmd/main/logo/compmd-dark.svg">
  <img src="https://raw.githubusercontent.com/comp-markdown/compmd/main/logo/compmd-light.svg" alt="CompMD" width="200">
</picture>

# @compmd/create-pack

Writes a CompMD pack: a component as one built ES module, with every library it uses bundled in, its signature, a test page that renders it in the strict sandbox as a host does, and the steps to publish it. The design is [docs/create-pack.md](../docs/create-pack.md).

**Website:** [compmd.hakanserce.com](https://compmd.hakanserce.com/), with the spec and a playground to try CompMD in.

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

It shares its version with the other `@compmd` packages, and the pack it writes pins `@compmd/vite`, `@compmd/processor`, `@compmd/sandbox`, Vite and Vitest at the versions released with it, so a pack made today builds the same later.

## What it writes

```
mermaid/
  package.json         "compmd": "./index.comp.md", the keyword compmd-pack; publishes dist/, index.comp.md and the README
  index.comp.md        the export, the import of ./dist/mermaid.js, the signature
  src/mermaid.ts       export default function mount(target, { params, fallback }) { … }
  src/mermaid.test.ts  the component in happy-dom
  demo/                the test page: a document that imports the pack, in the strict sandbox
  vite.config.ts       vite build: one module in dist/; vite: the test page, rebuilt on save
  scripts/check.js     before publishing: export.version is package.json's version
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

writes a library pack ([CompMD packs on a registry](../compmd-registry-v1.md), section 5): `src/index.ts` re-exports `three`, built as one module, `dist/index.js`, which `package.json` names in `"compmd": { "library": "dist/index.js" }`. It has no document, no component and no test page. Its version is the library's: the build writes the installed `three`'s version into `package.json`, and publishing checks it. Its README says when to add `"single": true`.

## Commands

```sh
pnpm install         # in the repository root
pnpm build           # the command into dist/
pnpm test            # writing packs, the template's components, written packs built and opened in Chrome and WebKit, and a library pack built
pnpm lint            # eslint and the type check, the template's files too
```

The template is in [`template/`](template): the files every pack has, those of a leaf or a code renderer, and a library pack's own. Its names are `__pack__`, `__component__`, `__language__` and `__library__`, which are valid in code, so its files are linted, type-checked and tested in place, but a library pack's `src/index.ts`, which imports the library only a written pack has.

## License

MIT.
