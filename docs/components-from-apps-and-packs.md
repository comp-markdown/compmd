# Components from apps and packs

**Status: proposed.**

A document names components; their code comes from somewhere. This design answers two questions that turn out to be one:

1. **An application has components of its own**, written in React or Svelte and built with the app (a rating widget, a map, a chart bound to the app's data). How does it let its documents use them?
2. **A pack ships code** for others to use: a chart kit, a Mermaid diagram component with a large library inside. How does that code reach a reader, and how fast?

Both are about who supplies a component's code, and so how much it is trusted, how it is loaded, and how authors write, share and reuse it.

## Decisions

1. **Two kinds of component code, told apart by who supplies it.**
   - **Document code** comes with a document: a fence in it, a component file next to it, or a pack it imports from a registry. Anyone may have written it, so it runs sandboxed (spec §15).
   - **Host code** is the application's own, built into the app. It runs in the app's page with the app's trust, as the renderer itself does. A document can use it but never supply it.
2. **A host component is a signature in a pack the host provides.** The host gives the processor an ambient pack (spec §6.3) whose document holds a signature fence alone for each of its components, and gives the renderer each component's code in `components`. This works in the processor today (shown below). The spec says so explicitly, and the packages make it short to write.
3. **Host code's inputs are untrusted.** A host component runs outside the sandbox, but its params and slots come from documents. The spec says what that means for the host's code (decision 7 below).
4. **A pack's code ships as it is written or built: no new format.** A component file in a pack is source (`.jsx`, `.svelte`, compiled in the sandbox) or a built ES module (`.js`, the plain binding, every dependency bundled in). Mermaid ships as one built module.
5. **Large pack code loads by URL, not as text.** Today a component file's text travels inside the processor's output. A built module of any size instead loads in the sandbox from the pack's files, served by the host from its own origin, checked against the pack's integrity, and cached by URL across documents and visits.
6. **Shared libraries come from the host, not the pack.** The sandbox's import map gives React, Svelte and the libraries the host chooses (the playground's: d3). A pack bundles what the host doesn't give it, so a pack never depends on a host's choice to run.
7. **The ambient pack's name is the host's.** A document may import it by name to say what it depends on (`from: "@acme/app"`); a processor resolves that name to the host's pack, never to a registry, as it does for `@compmd/*`.

## Today

- **What runs where.** The playground renders the page in a sandboxed iframe with an opaque origin and a content security policy that allows no requests (`default-src 'none'`). Document code reaches the frame as text: the processor's output carries each host-language component's code (`HostComponent.code`). The frame compiles it in the browser (sucrase for JSX and TSX, Svelte's compiler for Svelte), loads it as a `blob:` module, and mounts it through its binding (spec §7.6). Its imports resolve only through the frame's import map: React, React DOM, Svelte and d3.
- **What an app can do.** Pass the renderer `components: { stars: react(Stars) }`. But a document that writes `:: stars` gets F002 unless the processor knows the name. The way that works today is the one below, which no document says.

**A host component today**, checked with `@compmd/processor` 0.1.1. The host provides an ambient pack whose document exports a signature alone:

````md
---
md:
  export:
    pack: "@acme/app"
    version: "1.0.0"
    names: "*"
---

```compmd component=stars signature
params:
  value: number
```
````

The document `:: stars[4 of 5] {value=4}` then gives a `componentLeaf` named `stars`, with its params validated (`value="x"` is F009, `color=red` F008) and W113 for a name only this host has. `componentsOf` lists `stars` as kind `processor`, from the host's pack. The renderer mounts it from `components`. Nothing else is needed, except that writing a pack in JSON for each app is clumsy, and the spec's §15 says code runs sandboxed, which a host's own React component doesn't.

## The examples

Each alternative below is weighed on these.

1. **`stars`**, an app's own React component: a 5-star rating that shows a value and, bound to an input, sets it.
2. **`rating-summary`**, an app's own Svelte component that reads the app's data through `host` (spec §7.4).
3. **`@acme/charts`**, a pack of bar, line and pie charts in React and Svelte, published on npm and used by many documents in many apps.
4. **`@kit/mermaid`**, a pack with one component, `mermaid`, that draws a Mermaid diagram from a fenced block in its slot. Mermaid 11 is 3.57 MB as one ES module, 976 KB gzipped.

## The alternatives

### A. Keep today's way: an ambient pack with the component's code, mounted by the host

The host writes its component twice: as code in a `jsx` fence in an ambient pack, so the name resolves, and again in the app, mounted through `components`, which overrides the fence's code.

- **Security:** muddled. The fence's code is never run, but nothing says so; a reader of the pack sees code that isn't what renders.
- **Speed and size:** as B.
- **Experience:** every component written twice, and the two drift.

Rejected: it is B with a copy of the code that does nothing.

### B. Host components: a signature in a pack the host provides (recommended for apps)

The host declares each component by its signature, and implements it in the app:

```ts
import { hostPack, init, process } from "@compmd/processor";
import { createPage } from "@compmd/render";
import { react } from "@compmd/render/react";
import { Stars } from "./Stars";

await init();
const app = hostPack("@acme/app", { stars: "params:\n  value: number" });
const { expanded } = process({ path: "notes.comp.md", files, now, ...app });
createPage(element, { components: { stars: react(Stars) } }).render(expanded);
```

`hostPack(name, signatures)` writes the pack (its `package.json` and its document with the signature fences and the export) and gives `{ packs, ambient }` for the request. It is a few lines in `@compmd/processor`; the format is the existing one.

- **Security:**
  - *Who supplies the code:* the app; it runs in the app's page with the app's trust, like the renderer. A document only names it and gives it params and slots.
  - *What a document can make it do:* whatever its params allow. The host's code must treat params and slot content as untrusted input: no `innerHTML` from a param, URLs checked, actions outside the page only on the reader's action (spec §15, "Components the host trusts", applies, since these are components the host trusts).
  - *With the app's data:* `rating-summary` reads the app's data through `host`, with the reader's access. A shared document can aim it at data its author can't see, but the reader sees only what the reader may see; it is what a document rendered for someone else, such as a published site, must not be given (spec §15, "Reading for others").
  - *Impersonation:* a registry pack can't declare a host component: a signature fence alone is F002 in any pack the processor doesn't provide. A document can't either: a local definition named `stars` shadows the host's (spec §6.3) but is document code and runs sandboxed. With decision 7, a document importing `@acme/app` by name gets the host's pack, never a registry package of that name.
  - *Reporting:* W113 at each use, `componentsOf` lists it as the host's, and every other processor renders the fallback.
- **Speed and size:** nothing to download or compile: the component is in the app's bundle already, and mounts synchronously in the page, without the frame's message hop.
- **Experience:**
  - *Writing:* an ordinary component in the app's codebase, with its tests, plus a signature of a few YAML lines. The signature is the contract the processor checks documents against; the app's TypeScript types can be checked against it by a test.
  - *Sharing and reuse:* within the app, every document can use it. Outside, the document shows fallbacks (W113 warns the author). A document says what it depends on by importing the pack by name.

### C. Packs with source code (today's pack format; good for small components)

A pack's document imports component files (`./bar-chart.jsx`) or holds fences; the sandbox compiles them.

- **Security:** document code, sandboxed: no network, no session, only its params and slots (spec §15). Source is reviewable: the code that runs is the code in the package. Versions and `integrity` pin it (spec §6.1); npm provenance says where a version was built.
- **Speed and size:** each component's text travels through the processor's output on open; the frame loads a compiler the first time a document needs one (sucrase 43 KB gzipped for JSX and TSX, Svelte's compiler 221 KB), and compiles each component per open. Imports are limited to the import map. Fine for charts in plain SVG; impossible for Mermaid, whose library isn't in the map and is too large to pass as text.
- **Experience:**
  - *Writing:* a component file and a signature, tried live in the playground.
  - *Sharing:* `npm publish` of a folder with `package.json` and `index.comp.md`.
  - *Reuse:* `import: [{from: "@acme/charts@1", names: "*"}]`, in any processor that runs JavaScript; others show fallbacks.

### D. Packs with built modules, loaded by URL (recommended for large components)

A pack ships a component as one built ES module in the plain binding (`mermaid.js`, `export default function mount(target, context)`), every dependency bundled in, and its signature. The processor gives the renderer where the module is (the pack, its version, the file and the pack's integrity) instead of its text. The sandbox loads it by URL from the host's origin, which serves the pack's files.

````md
<!-- @kit/mermaid's index.comp.md -->
---
md:
  import:
    - from: ./mermaid.js
      names: [mermaid]
  export:
    pack: "@kit/mermaid"
    version: "1.0.0"
    names: [mermaid]
---

```compmd component=mermaid signature
slots: [default]
```
````

````md
<!-- a document -->
::: mermaid
```mermaid
graph LR; Draft --> Review --> Published
```
::: /mermaid
````

A plain renderer shows the fenced block, which GitHub draws as a diagram itself.

- **Security:** document code, sandboxed, as C. Built code is harder to review than source, so it rests more on provenance and integrity: the processor checks the module against the pack's integrity, and the content security policy allows scripts from the host's own origin only. The sandbox limits what any pack code can do, built or not: no network, so a compromised Mermaid can't send what it draws anywhere.
- **Speed and size:** no compiler. Mermaid's 976 KB load only for a document that uses it, once, and stay cached by URL across documents and visits, since a published version never changes. Nothing large passes through the processor. For comparison, the processor itself is 808 KB gzipped.
- **Experience:**
  - *Writing:* the author builds the module with their own bundler; the playground can't edit built code, only use it.
  - *Sharing and reuse:* as C, and any npm library can be used, bundled in.

### E. No declaration: the renderer's map alone

A host passes `components` and the processor accepts any name it doesn't know.

- Rejected: documents get no validation (F008, F009, F010), no fallbacks they can rely on, no hints; and every misspelt component name becomes a silent fallback instead of F002.

## Compared

| | A. code twice | B. host signature | C. pack source | D. pack built module | E. no declaration |
|---|---|---|---|---|---|
| Who supplies the code | the app | the app | the document or its pack | the pack | the app |
| Runs | in the page | in the page | in the sandbox | in the sandbox | in the page |
| Documents validated | yes | yes | yes | yes | no |
| Download for the reader | none | none | text + a compiler (43–221 KB) | the module, once, cached | none |
| Libraries | the app's | the app's | the import map's | anything, bundled | the app's |
| Works in another processor | fallback | fallback | yes, if it runs JavaScript | yes, if it runs JavaScript | fallback |
| Write it once | no | yes | yes | yes (and build it) | yes |

## What changes where

**The spec:**
- §6.3: an ambient pack's name resolves to the host's pack, never a registry (decision 7).
- §7.1: a signature fence alone in a pack the processor provides declares a component the host implements.
- §15: the sandbox rule covers document code; host code runs with the host's trust, and its inputs are untrusted, as for components the host trusts.
- §6.4 and §15: a pack's built module is loaded from the pack and checked against its integrity; the rule that a component has no network stays.

**The processor:** nothing for B, which works today. For D, `HostComponent` gives a built module's place (pack, version, path, integrity) instead of its text, when its file is a `.js` module in a pack.

**`@compmd/processor`:** `hostPack(name, signatures)`; `HostComponent`'s new form.

**`@compmd/render`:** nothing for B. The frame protocol takes a module's URL to load (D).

**The playground:** a host component example (`stars`), and loading built modules by URL; its frame's policy allows scripts from the site's own pack folder.

**The conformance set:** a case for a host signature in an ambient pack (W113, validation, a `componentLeaf`), and one for an ambient name imported explicitly.

## What this means

- **For apps:** B. The website's third snippet becomes the one above.
- **For the Mermaid pack:** D. It is a built module of the plain binding with Mermaid inside, loaded only when a document uses it. It can be written now as C, with its module as a component file whose text travels through the processor; D makes it fast.

## Not in this change

- A host component that runs in the sandbox (the host's own code, but untrusted by the host): a host that wants that writes a pack (C or D).
- Packs from registries other than npm.
- Lock files for packs (spec §6.1 leaves them to processors).
