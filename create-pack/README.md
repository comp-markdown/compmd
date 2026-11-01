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

It shares its version with the other `@compmd` packages, and the pack it writes pins `@compmd/vite`, `@compmd/processor`, `@compmd/sandbox`, Vite and Vitest at the versions released with it, so a pack made today builds the same later.

## What it writes

```
mermaid/
  package.json         "compmd": "./index.comp.md"; publishes dist/, index.comp.md and the README
  index.comp.md        the export, the import of ./dist/mermaid.js, the signature
  src/mermaid.ts       export default function mount(target, { params, fallback }) { … }
  src/mermaid.test.ts  the component in happy-dom
  demo/                the test page: a document that imports the pack, in the strict sandbox
  vite.config.ts       vite build: one module in dist/; vite: the test page, rebuilt on save
  scripts/check.js     before publishing: export.version is package.json's version
  README.md            what to change, how to try it, how to publish
```

The component is plain TypeScript in the plain binding (spec §7.6). React and Svelte stay out of the bundle, since the sandbox's frames provide them, so a component written in either renders it inside `mount`; the build adds each one it leaves out to `peerDependencies`, at the version it was built with, so a host with an older one reports it when it resolves the pack (spec §6.4).

## Commands

```sh
pnpm install         # in the repository root
pnpm build           # the command into dist/
pnpm test            # writing packs, the template's components, and a written pack built and opened in Chrome and WebKit
pnpm lint            # eslint and the type check, the template's files too
```

The template is in [`template/`](template): the files every pack has, and those of a leaf or a code renderer. Its names are `__pack__`, `__component__` and `__language__`, which are valid in code, so its files are linted, type-checked and tested in place.

## License

MIT.
