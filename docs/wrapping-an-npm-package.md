# Wrapping an npm package as a pack

**Status: approved (2026-11-20, CMD-8); built in CMD-9** (`@compmd/pack/wrap`, `create-pack --from`). Making a pack of an existing npm package should take one command, and moving
to the package's next version a version bump. This builds on [the pack template](create-pack.md) and
[writing a pack](writing-a-pack.md); CMD-9 builds it. [The spike](../spikes/wrap-npm-package/README.md) has the four
packs it was tried on, built and run.

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
   mounts the package's component in a box of its own in `target` (the renderer owns `target`'s classes and style)
   with the params as its props, makes its drawing fit that box ([sizing a component](sizing-components.md), decision 1),
   updates it, and unmounts it on `destroy`. A web component's glue passes its `error`
   event to `fail`, since such an element reports a failure without throwing.
   React, React DOM and Svelte stay external, from the frames' import map, so they are one copy whatever loads them
   (create-pack, decision 3). The command writes it once; it is plain code the author may change, and the build
   doesn't touch it again.
4. **The signature is generated at every build** (decided 2026-11-20). `compmd-pack build` reads the package's
   TypeScript types with the TypeScript checker and writes the signature fence of `index.comp.md` that stands between
   two comments, `<!-- compmd-pack:generated … -->` before it and `<!-- /compmd-pack:generated -->` after it, leaving
   the document's prose as it is (decided 2026-11-20, CR-29: comments, which need no spec change and which plain
   renderers hide; the opening one says what the fence is generated from and that edits to it are overwritten). The
   build fails on a pair with anything but one signature fence between, or on a comment without its pair. Removing
   the pair hands the fence to the author: the build then leaves it, and reports where it differs from the types. Updating the package to its next version is a
   version bump: the build writes the params its types now give, and says what changed (a param added to the package,
   one gone, one whose type changed), and a param that went fails the build where a demo or the overrides still name it.
5. **The params are an allowlist,** `params` in `compmd.wrap.json` beside `package.json`, which the command writes
   with every prop it could map, and the build keeps to: a package's new prop is reported, never added on its own.
   The wrap's settings are a file of their own, which the pack doesn't publish, because `package.json`'s `compmd`
   field is the pack's document already (companion `compmd-registry-v1.md`, sections 2 and 5). Only a prop declared
   in the package's own files counts (layerchart's `x`, from its `ChartProps`), never one from React's, Svelte's or
   the DOM's types (`SVGProps`, `HTMLAttributes`): react-qr-code would otherwise give several hundred.
6. **What maps, and how** (spec §7.3):

   | The prop's type | The param |
   |---|---|
   | `string`, `number`, `boolean`, `Date` | `string`, `number`, `boolean`, `date` |
   | a union of string literals | an enum (`L \| M \| H \| Q`) |
   | optional (`?`) | the type with `?` |
   | an array, a plain object | `array`, `object` |
   | a CSS colour type (`CSSProperties["color"]`) | `color` (spec §9.4) |
   | a CSS length type | `string` |
   | named like a common param (`height`, `width`, `background`, `border`, `padding`, spec §9.2) | none: the use's box takes it, and the component fits its box; listed (declaring one is F013, [sizing a component](sizing-components.md), decision 5) |
   | a size in pixels (react-qr-code's `size`) | none: the component fits its box, which the use sizes ([sizing a component](sizing-components.md), decision 1); listed |
   | a union with one usable branch (`string \| ((d) => any)`, `boolean \| Snippet`) | that branch, with a note in the build's report |
   | callbacks, `ref`, `style`, `className`, components, snippets with arguments | none: left out, and listed |

   A default comes from JSDoc `@default` only; a default written in a comment is never trusted (react-qr-code's says
   128, its code uses 256). Anything else is the overrides'.
7. **Overrides, in `compmd.wrap.json`:** a param's name (`fgColor` as `color`), its type, its default, its `doc`; which
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
    frames provide, and its CSS is bundled into the module (Svelte's `css: "injected"`; as a file of its own, the frame
    never loads it). A peer dependency beyond the runtimes (model-viewer's
    `three`) is bundled, or named as a library pack ([sharing library packs](sharing-library-packs.md)).

## In use

```sh
npm create @compmd/pack qr-code -- --from react-qr-code --name @comp-markdown/qr-code
```

writes the template's files ([the pack template](create-pack.md), *What it writes*), with `"react-qr-code": "2.2.0"`
in `dependencies` and ([the spike's pack](../spikes/wrap-npm-package/react-qr-code/), built and run):

```jsonc
// compmd.wrap.json; `size` is left out: the QR code fits its box, which the use sizes (decision 6)
{
  "from": "react-qr-code",
  "export": "default",
  "kind": "react",
  "params": ["value", "bgColor", "fgColor", "level", "title"],
  "label": "title"
}
```

```ts
// src/qr-code.ts, written once
import { createElement } from "react";
import { createRoot } from "react-dom/client";
import QRCode from "react-qr-code";
import { component } from "@compmd/pack/component";

/** The drawing fits its box, its aspect ratio kept (sizing a component, decision 1). */
const FILL = { display: "block", width: "100%", height: "100%" };

export default component<Params>((target, { params, fallback }) => {
  const root = createRoot(target);
  const draw = (next: Params) => {
    root.render(createElement(QRCode, { ...next, title: next.title ?? fallback.text, style: FILL }));
  };
  draw(params);
  return { update: draw, destroy: () => root.unmount() };
});
```

````md
<!-- compmd-pack:generated from react-qr-code 2.2.0's types and compmd.wrap.json; edits between these comments are overwritten at each build -->
```compmd component=qr-code signature
params:
  value: string
  bgColor: color?
  fgColor: color?
  level: L | M | H | Q?
  title: string?
```
<!-- /compmd-pack:generated -->
````

A document sizes it as any component: `:: qr-code[Our site] {value="https://compmd.dev/" width=10rem}`.

Moving to react-qr-code 2.3.0 is `npm install react-qr-code@2.3.0` and `npm run build`: the signature follows its
types, and the build lists what changed.

## Trying it on four packages

[The spike](../spikes/wrap-npm-package/README.md) read four packages' files and types, wrote by hand the packs the
command would write, built them as `compmd-pack build` does and opened each demo in strict frames, in Chrome and
WebKit (2026-11-20). Its findings are in decisions 3, 4, 5, 6 and 10, in gaps G6 to G8, and in [sizing a component in its
box](sizing-components.md).

| Package | Kind | What derives | What the author writes | In a strict frame |
|---|---|---|---|---|
| `react-qr-code` 2.2.0 (MIT) | React | 6 own props, cleanly, once `SVGProps`' are left out; 5 params, `size` left out for the box | the fallback; the drawing fitting its box | draws; 8.9 KB gzipped, React from the import map |
| `layerchart` 2.11.0 (MIT), `BarChart` | Svelte 5, ships `.svelte` sources | the scalar and enum props of about 90, with JSDoc defaults; not `height`, the box's | which of the 90 to keep; `x`, `y` as field names (their `string` branch); the theme's colours from `context.scheme` | draws, once compiled with its CSS injected; d3 and friends bundled, 164.5 KB gzipped |
| `@google/model-viewer` 4.4.0 (Apache-2.0) | web component, no `custom-elements.json` | about 50 properties, from the types | attribute names and defaults (they are in its sources, not its types); the slots | doesn't: the frame refuses the model's fetch and its WebAssembly; the glue shows the fallback, with W118 |
| `qrcode` 1.5.4 (MIT) | library, CommonJS | nothing | the glue: `toString(value, { type: "svg" })` into the target, and its params | draws; 9.3 KB gzipped |

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
- **G6, settled. An optional enum** (`level: L | M | H | Q?`): the spec's grammar puts `?` after a type, and an enum
  is one. CMD-9 adds it to the spec's example of declarations and a conformance case (`validate/optional-enum`).
- **G7, settled. How the generated fence is marked:** a word after `signature` in the info string would need the spec
  to say what other words there mean; comments before and after the fence need nothing (decision 4).
- **G8, settled by [sizing a component](sizing-components.md). A component's size when the use gives none** is its
  natural size, which for a drawing is seldom what an author wants: its decision 4 puts box defaults on the signature
  fence, a spec change. `compmd.wrap.json`'s `box` (`"box": "width=12rem max-width=100%"`) is the generated fence's
  param list, which the command writes empty and the author fills.

## What changes

- `@compmd/create-pack`: `--from`, the kind, the glue per kind, `compmd.wrap.json` with the allowlist, the warnings
  of decision 9.
- `@compmd/pack`: the build writes the generated signature (the TypeScript checker, the mapping, the overrides, the
  report of what changed), and the checks of decisions 9 and 10.
- [The pack template](create-pack.md), whose question C3 (a React or Svelte component) this answers, and
  [writing a pack](writing-a-pack.md).
- Not CMD-2: that is a package used as a library from a component's own code, with no pack; this makes a pack.
- Not the registry ([packs from a registry](packs-from-a-registry.md)): a wrapped pack is a pack like any other, built
  ahead into one module.
