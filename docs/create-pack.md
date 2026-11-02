# `@compmd/create-pack`, the pack template

**Status: accepted (2026-10-30); its build, checks and test page moved into `@compmd/pack` (2026-11-02).** Builds S2 of [components from apps and packs](components-from-apps-and-packs.md#s2-built-modules-loaded-by-url-recommended-for-large-components), its *step by step*: `npm create @compmd/pack mermaid` writes a pack whose component is one built ES module, with its libraries bundled in, that a host loads by URL. Its test page uses [`@compmd/vite`](vite.md), whose design is accepted.

[Writing a pack](writing-a-pack.md) changed where decisions 3, 5 and 7 live: the build, the publish check and the test page are `@compmd/pack`'s, run by the pack's scripts as `compmd-pack build`, `dev` and `test`, and are no longer copied into each pack. What they do is as below. The pack holds only what is its own.

## Proposed decisions

1. **One command, a few flags, no prompts.** `npm create @compmd/pack <folder> [-- --name @scope/pack] [--component name] [--renders language]`. Without `--name`, the pack is `@<your npm user>/<folder>` when `npm whoami` answers, else `@me/<folder>` with a note to change it. No questions asked, so it scripts and tests the same way it runs.
2. **The component is plain TypeScript in the plain binding** (spec §7.6): `src/<component>.ts` exports `mount(target, context)`. One template, not one per framework: the plain binding is what a built module is (spec §6.4), and a React or Svelte author renders their framework inside `mount`, as the template's README shows.
3. **The build is Vite's library mode**, writing `dist/<component>.js`: one ES module, every dependency bundled in, minified, with no `import` left but the shared runtimes (`react`, `react-dom`, `svelte`), which stay external because the frames' import map provides them (sandbox design Q4) and two copies of React break hooks. Which import names those are is `@compmd/sandbox`'s list (`RUNTIMES`), not a pattern, and the build adds each runtime it leaves out to `peerDependencies` at the version it was built with, so a host with an older one says so when it resolves the pack (spec §6.4, §7.6: [what component code may import](what-component-code-may-import.md)). Code splitting is off, so a library that loads parts of itself on demand stays in the one file, which a host that loads the module as text needs ([packs from a registry](packs-from-a-registry.md), R6, R7). The library packs the pack names in `peerDependencies` stay external too, and packs share them (R9).
4. **The pack's document imports the built file**: `index.comp.md` imports `./dist/<component>.js` and holds the signature, so what a host loads is what was published. `package.json` publishes `dist/`, `index.comp.md` and the README, nothing else.
5. **`npm run dev` opens a test page built on `@compmd/vite`**, with the pack as a folder pack: a `demo.comp.md` that uses the component, rendered in the strict sandbox exactly as a host renders it, rebuilt on save.
6. **`--renders <language>` makes it a code renderer** (spec §7.7): the signature says `renders: <language>`, `mount` reads `params.code`, and the demo writes a fenced block of that language.
7. **`--library <package>` makes a library pack** of an npm package (companion `compmd-registry-v1.md`, section 5): `src/index.ts` re-exports it, built as one module, `dist/index.js`, with `"compmd": { "library": "dist/index.js" }` and `exports` naming the same file. It has no document, no component and no test page, so `--library` refuses `--renders` and `--component`. **Its version is the library's**: the build writes the installed library's version into `package.json`, with a warning, as it writes the runtimes it imports into `peerDependencies`, and `prepublishOnly` checks they match. `"single": true` is the author's to add, for a library that can't have two copies in a sandbox; its README says when.
8. **A pack uses a library pack** by naming it in `peerDependencies` and importing it by that name; the pack's README gives the steps. Its test page, on `@compmd/vite`, gets the library only as one it offers (`libraries` in a `compmd.config.json` beside `demo.comp.md`, L1), since `@compmd/vite` serves built modules by URL ([sharing library packs](sharing-library-packs.md), decision 8).

## What it writes

For `npm create @compmd/pack mermaid -- --name @kit/mermaid --renders mermaid`:

```
mermaid/
  package.json        name, version 0.1.0, "compmd": "./index.comp.md", the keyword compmd-pack, files, scripts (compmd-pack), devDependencies
  index.comp.md       the export, the import of ./dist/mermaid.js, the signature
  src/mermaid.ts      export default component<Params>((target, context) => { … })
  src/mermaid.test.ts the component through the host contract, with mountForTest: it draws, updates, follows the scheme and is destroyed
  demo.comp.md        the test page's document: it imports the pack and uses it
  tsconfig.json       { "extends": "@compmd/pack/tsconfig.json", "include": ["src"] }
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
```
````

The component draws in the page's colour scheme, `context.scheme` (spec §7.4), rather than taking a theme param.

Without `--renders`, the signature has `params: { value: string }` and the demo a leaf, `:: mermaid[A fallback] {value="Hello"}`.

For `npm create @compmd/pack three -- --name @kit/three --library three`:

```
three/
  package.json        "compmd": { "library": "dist/index.js" }, exports, the keyword compmd-pack, files, three in devDependencies
  src/index.ts        export * from "three";
  tsconfig.json       @compmd/pack's
  README.md           how a pack uses it, when to add "single", how to publish
  .gitignore
```

**Scripts:** `build` (`compmd-pack build`: library mode, then the publish check), `dev` (`compmd-pack dev`, the test page), `test` (`compmd-pack test`, Vitest in happy-dom), `lint` (the type check), `prepublishOnly` (build and test), so `npm publish` never publishes a stale `dist/`. A library pack's `build` writes its library's version into `package.json` and checks it.

**Versions:** `export.version` must match `package.json` (F022). The template's README says to change both, and the build checks they match, failing with which to change.

## Questions, with proposed answers

**C1. Where the template lives.** Proposed: as files under `create-pack/template/`, copied with the names filled in, so the template is readable and its own files are linted and type-checked in the repository; tests generate a pack from it.

**C2. The versions it depends on.** Proposed: the generated `package.json` pins `@compmd/vite`, `@compmd/processor`, `@compmd/sandbox` and `vite` at the versions released with `create-pack`, written in at its build, so a pack made today builds the same next year.

**C3. A React or Svelte component.** Proposed: not in the first version (decision 2). If asked for, `--binding react` would write a JSX component and keep React external, as above.

**C4. Testing that a generated pack renders.** Proposed: a test that generates a pack into a temporary folder with the workspace's packages linked, builds it, checks `dist/` (one file, no bare import but the runtimes), and opens the test page in Chrome and WebKit (Vitest browser), where the component draws and its fallback text is its `aria-label`; the same for `--renders`.

## Not in this change

- Publishing for the author: the README gives the steps (`npm login`, `npm publish --access public`, provenance from CI).
- The playground loading the built pack.
- A test page that offers the library packs in `peerDependencies` by itself: the author lists them in a `compmd.config.json` beside `demo.comp.md` (decision 8).
