# 13. Your first pack

A pack is an npm package of components that any document can import, from any host. This chapter writes one with `npm create @compmd/pack`, opens its test page, and goes through the component it holds: one function in the plain binding, and the context the host gives it.

## Writing one

```sh
npm create @compmd/pack hello -- --name @you/hello
cd hello
npm install
npm run dev
```

The command asks nothing: everything is a flag, so it runs the same in a script. The flags go after `--`, which passes them on from `npm create`.

| Flag | What it does |
|---|---|
| `--name <@scope/pack>` | the npm package, which must be scoped; by default `@<your npm user>/<folder>`, or `@me/<folder>` when `npm whoami` doesn't answer, with a note to change it |
| `--component <name>` | the component's name, which documents use; by default the folder's name in lowercase, words joined by `-` |
| `--renders <language>` | makes the component a code renderer of that language's code blocks ([chapter 16](16-code-renderers.md)) |
| `--library <package>` | makes a library pack of an npm package instead, with no component ([chapter 15](15-react-svelte-and-libraries.md)) |

It refuses a folder that has files in it, a name that isn't scoped, and `--library` together with `--component` or `--renders`. When it is done it says what it wrote, here `Wrote @you/hello in hello: 8 files, with the component hello.`, and the three commands to run next.

## What it writes

Only what is the pack's own:

```text
hello/
  package.json         the package, its scripts, and what it publishes
  index.comp.md        the pack's document: the export, the import of the built module, the signature
  src/hello.ts         the component
  src/hello.test.ts    its tests
  demo.comp.md         the test page's document
  tsconfig.json        one line: @compmd/pack's
  README.md            what to change, how to try it, how to publish
  .gitignore           node_modules and dist
```

There is no build configuration, no test page and no publish check in it. Those are [`@compmd/pack`](../../pack/README.md)'s, and the scripts run its command, `compmd-pack`, so upgrading `@compmd/pack` updates them in every pack made earlier:

```json snippet=first-pack/package.json
{
  "name": "@you/hello",
  "version": "0.1.0",
  "description": "A CompMD pack: hello.",
  "keywords": ["compmd", "compmd-pack"],
  "license": "MIT",
  "type": "module",
  "compmd": "./index.comp.md",
  "files": ["dist", "index.comp.md", "README.md"],
  "scripts": {
    "build": "compmd-pack build",
    "dev": "compmd-pack dev",
    "test": "compmd-pack test",
    "lint": "tsc --noEmit",
    "prepublishOnly": "compmd-pack build && compmd-pack test"
  },
  "devDependencies": {
    "@compmd/pack": "0.3.0",
    "happy-dom": "21.0.3",
    "typescript": "6.0.3",
    "vitest": "5.0.5"
  }
}
```

The versions in `devDependencies` are the ones `@compmd/create-pack` was released with, pinned, so a pack made today builds the same later. [Chapter 17](17-testing-and-publishing.md) goes through the fields that make it a pack on a registry.

The pack's document is what a host reads. It imports the component's built module, `./dist/hello.js`, exports it under the pack's name and version, and declares its signature, which every use is checked against before anything runs:

````md snippet=first-pack/index.comp.md
---
md:
  import:
    - from: ./dist/hello.js
      names: [hello]
  export:
    pack: "@you/hello"
    version: "0.1.0"
    names: [hello]
---

# @you/hello

`hello`, a leaf: `:: hello[What a reader without it sees] {value="Hello"}`.

```compmd component=hello signature
params:
  value: string
```
````

The import names the built file, not the source: what a host loads is what you published. `compmd-pack build` builds `dist/hello.js` from `src/hello.ts` (or `.tsx`, `.js`, `.jsx`), one module for each `./dist/<name>.js` the document imports. [Chapter 14](14-signatures-and-params.md) is about the signature.

The TypeScript configuration is `@compmd/pack`'s, strict, with JSX for React:

```json snippet=first-pack/tsconfig.json
{ "extends": "@compmd/pack/tsconfig.json", "include": ["src"] }
```

## The test page

`npm run dev` runs `compmd-pack dev`, which builds the pack's modules and serves a page that renders `demo.comp.md`. The template writes one that imports the pack and uses its component:

```text
---
md:
  import:
    - from: "@you/hello@0"
      names: [hello]
---

# hello

:: hello[Hello, from hello] {value="Hello"}
```

The page renders it as a host renders it: the pack as a folder pack, in host mode, each use in a strict frame of its own that takes the height of what it draws. Under the page are the document's diagnostics and the components' failures, and above it a switch between the reader's colour scheme, light and dark. Saving a file in `src/` (but a test) builds the modules again, and the page reloads. `--port <n>` picks the port: `npm run dev -- --port 4300`.

Nothing of the page is written into the pack. Add uses to `demo.comp.md` as you add params, so you see each one, light and dark. The demo isn't published: `files` doesn't list it.

## The plain binding

A built module is the plain binding (spec [§7.6](https://compmd.hakanserce.com/spec/#76-bindings)): its default export *is* `mount`, a function that gets an element and a context, draws into the element, and returns what to do on an update and when the component goes. The template's component:

```ts snippet=first-pack/src/hello.ts
// `hello`: draws its `value` in its box. Replace the drawing with yours; what it imports is bundled in.
import { component } from "@compmd/pack/component";

/** The params, as the signature in `index.comp.md` declares them. */
export interface Params {
  value: string;
}

export default component<Params>((target, { params, fallback }) => {
  const shown = document.createElement("p");
  // The fallback is what a reader without the component sees: its text is what you draw, as text.
  shown.setAttribute("aria-label", fallback.text);
  shown.style.cssText = "display: inline-block; margin: 0; padding: 0.4em 0.9em; border: 2px solid currentColor; border-radius: 999px; font-weight: 600;";
  target.append(shown);
  const draw = ({ value }: Params) => {
    shown.textContent = value;
  };
  draw(params);
  return {
    update: draw,
    destroy: () => {
      shown.remove();
    },
  };
});
```

`component<Params>` from `@compmd/pack/component` only types it: it returns your function as it is, a `Mount`, with `params` and `update`'s params typed as `Params` instead of `Record<string, unknown>`. The interface repeats the signature; the signature is what counts, and the processor has checked every value against it before the component sees it. Nothing generates one from the other yet, so keep them the same.

`target` is the component's box, which the host made: it has already applied the common params a document gives (`width`, `padding`, `border`, …), `class`, `id`, `lang` and `dir`, and the component doesn't receive them. What is inside the box is the component's.

## The context

The second argument is the host contract's context (spec [§7.4](https://compmd.hakanserce.com/spec/#74-the-host-contract)):

| Field | What it is |
|---|---|
| `params` | the params as the signature declares them, checked and typed, with the defaults of those the document didn't give |
| `slots` | each slot's content by name, `default` too: a list of items, each `{ element, component? }`, which the host rendered and the component places ([chapter 14](14-signatures-and-params.md#slots)) |
| `set(param, value)` | changes the input bound to an `input<…>` param ([chapter 14](14-signatures-and-params.md#inputs-and-set)); it throws for any other param, and during `update` |
| `fallback` | the use's fallback, `{ text, element }`: as plain text, the component's text alternative, and rendered |
| `headingLevel` | the level the component's top heading should have, so the page's outline stays right wherever it is used |
| `scheme` | `"light"` or `"dark"`: the colour scheme the page shows the component in |
| `fail(message)` | the component can't draw what it was given, found after `mount` or `update` returned |
| `host` | what the host gives the components of packs it makes permissive; absent otherwise, and always in the strict sandbox |

**`fallback.text`** is what a plain renderer shows in the component's place: `Hello, from hello` in the demo. A component that draws anything but text (a chart, a map, an icon) uses it as its text alternative, as `hello` does with `aria-label`; the host doesn't label the box, since only the component knows its role.

**`scheme`** is the page's, not the reader's preference, which may differ: a light page for a reader who prefers dark is light. A component styled with CSS colours (`currentColor`, as `hello` is) needs nothing. One that takes its colours when it draws reads `scheme`, and when the scheme changes it gets an update with the same params.

**`fail(message)`** is for a component that finds out late that it can't draw what it was given: an asynchronous parse, say. The host reports W118 with the message, and shows the fallback in the box until the next update, when it empties the box and gives it back. It is for input the component can't draw, not for a resource that may come back: a map whose tiles stop loading shows its own state and retries. [Chapter 15](15-react-svelte-and-libraries.md#drawing-asynchronously) has a helper that calls it for you.

## Update and destroy

`mount` runs once per use. It returns an object with two optional functions, or nothing at all when it needs neither:

- **`update(params, slots)`** is called when a param or a slot changes (a reader changed an input, the data changed), and when the scheme changes, with the same params. The component is the same instance, and keeps its state: update what you drew, don't draw it again from nothing. Read the new values from the arguments; `context.params` and `context.slots` are those of the mount, while `context.scheme` is always the current one.
- **`destroy()`** is called before the box is removed: stop timers, remove listeners, unmount a framework.

A throw in `mount` shows the fallback for good, with W118: the component never mounted, so there is nothing to update. A throw in `update` is the same as `fail`: the fallback shows until the next update.

## Its tests

The template writes a test that runs the component as a host does, with `mountForTest` from `@compmd/pack/testing`:

```ts snippet=first-pack/src/hello.test.ts
import { mountForTest } from "@compmd/pack/testing";
import { describe, expect, it } from "vitest";
import mount from "./hello.ts";

describe("hello", () => {
  it("draws its value, follows new params, and goes when destroyed", () => {
    const shown = mountForTest(mount, { params: { value: "Hello" }, fallback: "Hello, as text" });
    expect(shown.target.textContent).toBe("Hello");
    expect(shown.target.querySelector("[aria-label]")?.getAttribute("aria-label")).toBe("Hello, as text");
    shown.update({ value: "Again" });
    expect(shown.target.textContent).toBe("Again");
    shown.destroy();
    expect(shown.target.textContent).toBe("");
  });
});
```

`npm test` runs it in happy-dom; `npm run build` builds `dist/hello.js` and checks the pack as a host would. Both, and the rest of what `mountForTest` gives, are in [chapter 17](17-testing-and-publishing.md).

Next: [signatures and params](14-signatures-and-params.md), what a component can declare and how a document's use of it is checked.
