+++
id = "01a1fe67-7944-7232-9b31-9af6149afe05"
number = 43
title = "Wrap an npm package as a pack in one command"
status = "approved"
tasks = ["CMD-9"]
author = "claude:79612601"
reviewers = []
created = 2026-11-21T13:59:16Z
updated = 2026-11-21T14:24:50Z
+++

[Wrapping an npm package as a pack](docs/wrapping-an-npm-package.md), built: `npm create @compmd/pack <folder> -- --from <package>` writes the pack, and `compmd-pack build` writes its signature from the package's types at each build. **The spec gains one example line (G6), which needs your go before it is pushed.**

## `@compmd/pack`: the wrap at build time (`src/wrap/`, exported as `@compmd/pack/wrap`)

One concern per file:
- `config.ts`: `compmd.wrap.json`, read and checked (decisions 5, 7).
- `props.ts`: the props, read with the TypeScript checker from a file that imports the component and names `ComponentProps<typeof …>` (React, Svelte) or `HTMLElementTagNameMap[tag]` (a web component). Only the package's own declarations count, its folder and its `@types` package's, never React's or the DOM's. The colour type is read from the declaration as written (`CSSProperties["color"]`), since the alias is gone once `undefined` joins it. `@default` comes from the declaration's JSDoc. `elementTags` lists a package's custom elements.
- `map.ts`: decision 6's table. Left out, with why: callbacks, `ref`, `key`, `style`, `className`, `children` unless it is a slot, any name in `BOX_PARAMS` (the box takes it), and a `size` that is a number. A union takes its one usable branch, with a note. Overrides go over the result, and `input` makes `input<…>` with no default.
- `signature.ts`: the fence, in the short form, or the long form for a param with a doc, through `yaml`. The slots, and `box` as the fence's param list (G8).
- `generated.ts`: the comments pair (decision 4): found, written again with the rest of the document kept; anything but one pair around one fence fails the build.
- `changes.ts`: what changed since the last fence, and a param that went still named in `demo.comp.md` or the overrides, which fails the build.
- `risks.ts`: decision 9 in the built module: a URL of another origin (namespaces such as SVG's aren't fetches), a worker, `eval` or `new Function`, WebGL.
- `generate.ts`: the build's step. Without the comments the fence is the author's: left as it is, with where the types differ, or why they can't be read.
- `kind.ts`, `installed.ts`: decision 2's kind, and a package found as Node finds it.

`build.ts`: `buildModules` runs the wrap first, compiles a Svelte package with `@sveltejs/vite-plugin-svelte` (`emitCss: false`, `css: "injected"`, decision 10), returns the notes, which `compmd-pack build` and `dev` print, and checks the modules. **A fix found on the way:** the manifest step read a runtime's version only from the pack's own `node_modules`, so a pack in a workspace, with its packages hoisted, failed to build; it now finds them as Node does.

`typescript` and `@sveltejs/vite-plugin-svelte` are dependencies now, `svelte` an optional peer.

## `@compmd/create-pack`: `--from`, `--export`, `--kind`

`src/wrap.ts`, `createWrappedPack`:
1. It writes the template's common files and installs the package (`npm install`, an option for tests), then pins it at the exact version installed (decision 8).
2. It reads the kind, adds the development packages the kind's glue needs (React and its types, or Svelte), and installs again for them.
3. It writes the glue (`template/wrap/<kind>/`), `compmd.wrap.json` with every prop that maps, the document with its generated fence, the demo, a test, and its own README.
4. It lists what it left out, and what in the package's entry file a strict frame refuses.

`create.ts` lends it `writePart`, `mustBeEmpty` and `versions`, which now reads `dependencies` too, since `@compmd/pack` is one now that the command imports `@compmd/pack/wrap`.

## Tests

- Unit tests for each file above, on fixture packages with hand-written types (`test/wrapped.ts`).
- A build test that compiles a Svelte component's sources, with its CSS in the module.
- `create-pack`'s unit tests: a web component, a React one, a plain library, and what is refused.
- **`test/pack.test.ts` wraps the real `react-qr-code` 2.2.0**, then type-checks, tests and builds it, and draws its demo in the strict sandbox in Chrome and WebKit, its fallback as the SVG's `<title>`. What it writes is the design's own example, `level: L | M | H | Q?` and all.

## Three choices to check

1. **The command installs the package itself.** It has to, to read the package's types and kind, and decision 1 says it asks nothing. A second install follows only when the kind needs React or Svelte.
2. **The glue reads its props' type from the package** (`ComponentProps<typeof Wrapped>`), not from a `Params` interface written once, so it can't fall behind the generated signature.
3. **"Else the only one" of decision 1 is done for web components only** (their only custom element). For React and Svelte, the default export is taken, or `--export` names one; reading which exports are components would need a guess at what a component is.

## The spec and the docs

- §7.3's example of declarations shows `level: L | M | H | Q?`; `conformance/validate/optional-enum` (an optional enum left out, given, and given a member it hasn't: F009). The changelog says it is no change. G6 is settled in the design.
- The design's status, `docs/create-pack.md` (decision 9), `docs/writing-a-pack.md` (decision 2), and the READMEs of `create-pack` and `pack`.
