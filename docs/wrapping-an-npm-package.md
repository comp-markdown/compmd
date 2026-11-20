# Wrapping an npm package as a pack

**Status: proposed (2026-11-20, CMD-8).** Making a pack of an existing npm package should take one command, and moving
to the package's next version a version bump. This builds on [the pack template](create-pack.md) and
[writing a pack](writing-a-pack.md); CMD-9 builds it.

## Decisions

1. **One command, beside the template's others:** `npm create @compmd/pack <folder> -- --from <package>[@<version>]
   [--export <name>] [--name @scope/pack] [--component <name>]`. It asks nothing, as the template doesn't (create-pack,
   decision 1). `--export` picks the package's export when it has more than one component (`BarChart`); by default its
   default export, else the only one.
2. **Two kinds, both served** (decided 2026-11-20):
   - **A ready-made component** (React, Svelte 5, or a web component): a complete pack. Its props become the
     signature's params, and a demo uses it. The author writes the fallback and reads what was derived.
   - **A plain library** (Observable Plot, gridjs, qrcode, abcjs, what most of [the first components](first-components.md)
     wrap): the pack with the library installed and bundled, and a `draw()` skeleton that imports it
     ([writing a pack](writing-a-pack.md), decision 6), for the author to finish, as `create-pack`'s leaf template is.

   The kind is read from the package: `react` in its peer dependencies is React; `svelte` there, or a `svelte` export
   condition, is Svelte; a custom element in its types (`HTMLElementTagNameMap`) or a `custom-elements.json` is a web
   component; anything else is a library. `--kind` overrides it.
3. **The glue is the pack's own code, in the plain binding** (spec §7.6), as every built module is: `src/<component>.ts`
   mounts the package's component in `target` with the params as its props, updates it, and unmounts it on `destroy`.
   React, React DOM and Svelte stay external, from the frames' import map, so they are one copy whatever loads them
   (create-pack, decision 3). The command writes it once; it is plain code the author may change, and the build
   doesn't touch it again.
4. **The signature is generated at every build** (decided 2026-11-20). `compmd-pack build` reads the package's
   TypeScript types with the TypeScript checker and writes the signature fence of `index.comp.md`, the one marked
   `generated` in its info string, leaving the document's prose as it is. Updating the package to its next version is a
   version bump: the build writes the params its types now give, and says what changed (a param added to the package,
   one gone, one whose type changed), and a param that went fails the build where a demo or the overrides still name it.
5. **The params are an allowlist** in `package.json`, `compmd.wrap.params`, which the command writes with every prop
   it could map, and the build keeps to: a package's new prop is reported, never added on its own. Only a prop's own
   declaration counts, never what its props type inherits (`SVGProps`, `HTMLAttributes`): react-qr-code would
   otherwise give several hundred.
6. **What maps, and how** (spec §7.3):

   | The prop's type | The param |
   |---|---|
   | `string`, `number`, `boolean`, `Date` | `string`, `number`, `boolean`, `date` |
   | a union of string literals | an enum (`L \| M \| H \| Q`) |
   | optional (`?`) | the type with `?` |
   | an array, a plain object | `array`, `object` |
   | a CSS colour or length type (`CSSProperties["color"]`) | `string` |
   | a union with one usable branch (`string \| ((d) => any)`, `boolean \| Snippet`) | that branch, with a note in the build's report |
   | callbacks, `ref`, `style`, `className`, components, snippets with arguments | none: left out, and listed |

   A default comes from JSDoc `@default` only; a default written in a comment is never trusted (react-qr-code's says
   128, its code uses 256). Anything else is the overrides'.
7. **Overrides, in `compmd.wrap`:** a param's name (`fgColor` as `color`), its type, its default, its `doc`; which
   prop is an input, with the callback that reports it (`"input": { "value": "onChange" }` makes `value` an
   `input<…>`); which prop takes the fallback's text when the package has one (a `title`, an `alt`); and which plain
   children are a slot (`"slots": { "default": "children" }`), since snippets and children that take arguments
   rarely are Markdown. A web component's attribute names are its properties' in kebab case unless a
   `custom-elements.json` says otherwise or the overrides do.
8. **The pack has its own version** (decided 2026-11-20): `0.1.0`, `0.2.0` and on, apart from the package's. It
   names the package at an exact version in its `dependencies` and in its README, so a pack's version always means
   one build of the package; moving to the package's next version is a release of the pack.
9. **What can't run in a strict frame is said at once.** The command and the build look for what the frame refuses
   (spec §15): a URL to a CDN or another origin in the bundle, a `new Worker`, `eval` or `new Function`, WebGL, and a
   component whose main input is a URL (a model, a video). Each is a warning naming where it is, and the README the
   command writes says what the pack then needs: its assets in the pack ([the first components](first-components.md),
   decision 5), a permissive level with the origins it reaches, or another package.
10. **The build makes one module of any package:** CommonJS is converted, dynamic imports are inlined (code splitting
    is off already, create-pack decision 3), a package that ships `.svelte` sources is compiled with the Svelte the
    frames provide, and its CSS is bundled into the module. A peer dependency beyond the runtimes (model-viewer's
    `three`) is bundled, or named as a library pack ([sharing library packs](sharing-library-packs.md)).

## In use

```sh
npm create @compmd/pack qr-code -- --from react-qr-code --name @comp-markdown/qr-code
```

writes the template's files ([the pack template](create-pack.md), *What it writes*), with:

```json
"dependencies": { "react-qr-code": "2.2.0" },
"compmd": {
  "wrap": {
    "from": "react-qr-code",
    "export": "default",
    "params": ["value", "size", "bgColor", "fgColor", "level", "title"],
    "label": "title"
  }
}
```

```ts
// src/qr-code.ts, written once
import { createElement } from "react";
import { createRoot } from "react-dom/client";
import QRCode from "react-qr-code";
import { component } from "@compmd/pack/component";

export default component((target, { params, fallback }) => {
  const root = createRoot(target);
  const draw = (next: typeof params) => root.render(createElement(QRCode, { ...next, title: next.title ?? fallback.text }));
  draw(params);
  return { update: draw, destroy: () => root.unmount() };
});
```

````md
<!-- index.comp.md: the fence below is rewritten at each build -->
```compmd component=qr-code signature generated
params:
  value: string
  size: number?
  bgColor: string?
  fgColor: string?
  level: L | M | H | Q?
  title: string?
```
````

Moving to react-qr-code 2.3.0 is `npm install react-qr-code@2.3.0` and `npm run build`: the signature follows its
types, and the build lists what changed.

## Trying it on four packages

A spike unpacked four packages (`npm pack`, nothing run) and read their files and types (2026-11-20):

| Package | Kind | What derives | What the author writes | In a strict frame |
|---|---|---|---|---|
| `react-qr-code` 2.2.0 (MIT) | React | 6 own props, cleanly, once `SVGProps`' are left out | the fallback; the default `size` (its comment is wrong) | runs: SVG, no network |
| `layerchart` 2.11.0 (MIT), `BarChart` | Svelte 5, ships `.svelte` sources | the scalar and enum props of about 90, with JSDoc defaults | which of the 90 to keep; `x`, `y` as field names (their `string` branch); the theme's colours from `context.scheme` | runs, once compiled and its dynamic imports inlined; d3 and friends bundled |
| `@google/model-viewer` 4.4.0 (Apache-2.0) | web component, no `custom-elements.json` | about 50 properties, from the types | attribute names and defaults (they are in its sources, not its types); the slots | doesn't: its model is a URL, its decoders load from CDNs, it needs WebGL and `blob:` workers |
| `qrcode` 1.5.4 (MIT) | library, CommonJS | nothing | the glue: `toString(code, options)` into the target, and its params | runs |

## Gaps

- **G1. A component that needs the network or the page's files** (model-viewer) can't be wrapped as a strict pack.
  Decision 9 says so; running it waits for a component reading the document's files (CMD-266) and WebGL in the
  frames ([the first components](first-components.md), *Later*).
- **G2. Generic props** (layerchart's `BarChart<TData>`) resolve with their constraint (`any`), which maps to
  `array` or `object`, never to the rows' fields.
- **G3. A theme's colours** come from CSS variables a package defines (`--color-primary`); mapping them to
  CompMD's visual tokens and `context.scheme` is the glue's, by hand.
- **G4. A Svelte package's sources must compile with the frames' Svelte:** one that needs a newer Svelte than the
  frames' fails the build, with which it needs.
- **G5. The checker costs each build** a TypeScript program over the package's types, not measured yet; only the
  build pays it, never a reader.
- **G6. An optional enum** (`level: L | M | H | Q?`): the spec's grammar puts `?` after a type, and an enum is one,
  but neither the spec nor a conformance case shows one. To confirm in the spec before the build writes it.

## What changes

- `@compmd/create-pack`: `--from`, the kind, the glue per kind, `compmd.wrap` with the allowlist, the warnings of
  decision 9.
- `@compmd/pack`: the build writes the generated signature (the TypeScript checker, the mapping, the overrides, the
  report of what changed), and the checks of decisions 9 and 10.
- [The pack template](create-pack.md), whose question C3 (a React or Svelte component) this answers, and
  [writing a pack](writing-a-pack.md).
- Not CMD-2: that is a package used as a library from a component's own code, with no pack; this makes a pack.
- Not the registry ([packs from a registry](packs-from-a-registry.md)): a wrapped pack is a pack like any other, built
  ahead into one module.
