# `@compmd/create-pack`, the pack template

**Status: proposed.** Builds S2 of [components from apps and packs](components-from-apps-and-packs.md#s2-built-modules-loaded-by-url-recommended-for-large-components), its *step by step*: `npm create @compmd/pack mermaid` writes a pack whose component is one built ES module, with its libraries bundled in, that a host loads by URL. Its test page uses [`@compmd/vite`](vite.md), whose design is in review alongside.

## Proposed decisions

1. **One command, a few flags, no prompts.** `npm create @compmd/pack <folder> [-- --name @scope/pack] [--component name] [--renders language]`. Without `--name`, the pack is `@<your npm user>/<folder>` when `npm whoami` answers, else `@me/<folder>` with a note to change it. No questions asked, so it scripts and tests the same way it runs.
2. **The component is plain TypeScript in the plain binding** (spec §7.6): `src/<component>.ts` exports `mount(target, context)`. One template, not one per framework: the plain binding is what a built module is (spec §6.4), and a React or Svelte author renders their framework inside `mount`, as the template's README shows.
3. **The build is Vite's library mode**, writing `dist/<component>.js`: one ES module, every dependency bundled in, minified, with no `import` left but the shared runtimes (`react`, `react-dom`, `svelte`), which stay external because the frames' import map provides them (sandbox design Q4) and two copies of React break hooks.
4. **The pack's document imports the built file**: `index.comp.md` imports `./dist/<component>.js` and holds the signature, so what a host loads is what was published. `package.json` publishes `dist/`, `index.comp.md` and the README, nothing else.
5. **`npm run dev` opens a test page built on `@compmd/vite`**, with the pack as a folder pack: a `demo.comp.md` that uses the component, rendered in the strict sandbox exactly as a host renders it, rebuilt on save.
6. **`--renders <language>` makes it a code renderer** (spec §7.7): the signature says `renders: <language>`, `mount` reads `params.code`, and the demo writes a fenced block of that language.

## What it writes

For `npm create @compmd/pack mermaid -- --name @kit/mermaid --renders mermaid`:

```
mermaid/
  package.json        name, version 0.1.0, "compmd": "./index.comp.md", files, scripts, devDependencies
  index.comp.md       the export, the import of ./dist/mermaid.js, the signature
  src/mermaid.ts      export default function mount(target, { params, fallback }) { … }
  src/mermaid.test.ts the mount in happy-dom: it draws, updates and is destroyed
  demo/index.html     the test page
  demo/demo.comp.md   a document that imports the pack and uses it
  demo/main.ts        the processor and the sandbox, from virtual:compmd
  vite.config.ts      library mode for the build; @compmd/vite for the test page
  tsconfig.json
  README.md           what to change, how to try it, how to publish
  .gitignore
```

````md
<!-- index.comp.md -->
---
md:
  import:
    - from: ./dist/mermaid.js
      names: [mermaid]
  export:
    pack: "@kit/mermaid"
    version: "0.1.0"
    names: [mermaid]
---

```compmd component=mermaid signature
renders: mermaid
params:
  theme: default | dark = default
```
````

Without `--renders`, the signature has `params: { value: string }` and the demo a leaf, `:: mermaid[A fallback] {value="Hello"}`.

**Scripts:** `build` (library mode), `dev` (the test page), `test` (Vitest), `prepublishOnly` (build and test), so `npm publish` never publishes a stale `dist/`.

**Versions:** `export.version` must match `package.json` (F022). The template's README says to change both, and `prepublishOnly` checks they match, failing with which to change.

## Questions, with proposed answers

**C1. Where the template lives.** Proposed: as files under `create-pack/template/`, copied with the names filled in, so the template is readable and its own files are linted and type-checked in the repository; tests generate a pack from it.

**C2. The versions it depends on.** Proposed: the generated `package.json` pins `@compmd/vite`, `@compmd/processor`, `@compmd/sandbox` and `vite` at the versions released with `create-pack`, written in at its build, so a pack made today builds the same next year.

**C3. A React or Svelte component.** Proposed: not in the first version (decision 2). If asked for, `--binding react` would write a JSX component and keep React external, as above.

**C4. Testing that a generated pack renders.** Proposed: a test that generates a pack into a temporary folder with the workspace's packages linked, builds it, checks `dist/` (one file, no bare import but the runtimes), and opens the test page in Chrome and WebKit (Vitest browser), where the component draws and its fallback text is its `aria-label`; the same for `--renders`.

## Not in this change

- Publishing for the author: the README gives the steps (`npm login`, `npm publish --access public`, provenance from CI).
- The playground loading the built pack.
- Libraries from the registry instead of bundled (S3, still open).
