# 2. Quick start

A Vite app that renders a CompMD document, with an input the reader changes. Four files, about forty lines.

## Install

```sh
npm create vite@latest my-pages -- --template vanilla-ts
cd my-pages
npm install @compmd/vite @compmd/processor @compmd/sandbox
```

## The plugin

`@compmd/vite` reads your configuration, gives the processor your packs, and serves the sandbox's frame documents from your app:

```ts snippet=quick-start/vite.config.ts
import { compmd } from "@compmd/vite";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [compmd()],
});
```

## A document

Save this as `notes.comp.md`. Its frontmatter has a list of tasks and an input, `done`, which the number input changes:

```md snippet=quick-start/notes.comp.md
---
tasks: [Write the outline, Draft the first part, Ask for a review, Publish]
md:
  inputs:
    done: number = 1
---

# The guide, step by step

::: box {padding=m border=subtle border-radius=m}
**:value[1 of 4 done]{of=(string(md.input.done) + " of " + string(size(md.doc.tasks)) + " done")}**

:: number-input[Done: 1] {bind=done}
::: /box

::: each {in=(md.doc.tasks) as=task join=list}
1. {{task}}
::: /each
```

[Try it](https://compmd.hakanserce.com/playground/?doc=guides/developers/quick-start) in the playground.

## The page

A place for the page in `index.html`:

```html snippet=quick-start/index.html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <title>My first CompMD page</title>
  </head>
  <body>
    <main id="page" style="height: 24rem"></main>
    <script type="module" src="./main.ts"></script>
  </body>
</html>
```

And `main.ts`, which opens the document and renders it:

```ts snippet=quick-start/main.ts
import { init, OpenDocument } from "@compmd/processor";
import { createSandbox } from "@compmd/sandbox";
import { request, sandbox } from "virtual:compmd";
import text from "./notes.comp.md?raw";

await init();

// The document, open with what the app gives it: its packs, from compmd.config.json.
let inputs: Record<string, unknown> = {};
const open = () =>
  new OpenDocument({ ...request, path: "notes.comp.md", files: { "notes.comp.md": text }, now: new Date().toISOString(), inputs });
let opened = open();

// Its page, in a sandboxed frame.
const page = document.getElementById("page") as HTMLElement;
const view = createSandbox(sandbox).open(page, { components: opened.components });
view.render(opened.expanded);

// A reader changed an input: open the document again with its new value.
view.on("set", ({ input, value }) => {
  inputs = { ...inputs, [input]: value };
  opened.free();
  opened = open();
  view.render(opened.expanded);
});
```

For `virtual:compmd`'s types, add `"@compmd/vite/client"` to `compilerOptions.types` in `tsconfig.json`.

## Run it

```sh
npm run dev
```

The page shows the heading, a box with *1 of 4 done* and the number input, and the four tasks. Set the input to 3, and the box says *3 of 4 done*: the frame tells your page the input changed, and your page opens the document again with the new value.

## What just happened

- `init()` loaded the processor, as WebAssembly.
- `OpenDocument` read the document with what `request` holds (your packs, none yet), and gave its expanded tree and the components it uses. The fields `main.ts` adds are the document's own: `path`, the document's name in `files`, which holds its text by path; `now`, the time the page is made, which `md.today()` reads, since the processor has no clock of its own; and `inputs`, the inputs' values by name.
- The frontmatter's `md.inputs` declared the input, `done: number = 1`: its type and the value it has until a reader changes it. `md.input.done` reads it in an expression, and `bind=done` ties the number input to it. A value in `inputs` that doesn't fit the input's type is ignored, and the default is used.
- `createSandbox(sandbox)` made a strict frame for the page, from the frame documents of the bundle the plugin builds and serves under `compmd/` (and writes into the app's output when it builds), and rendered the tree in it.
- The number input is a built-in component; when the reader changed it, the frame sent `set`, and nothing else can: the page's code runs in a frame with no network and no access to your page.
- Opening again on each change is the simplest way to keep the page in step, and fast enough for a document this size. [Live editing](05-live-editing.md) does it with less work, for an editor.

## When something is wrong

Nothing in the document is ever an exception: a mistake in it is a diagnostic, and the page still renders, with fallbacks where it must. `opened.diagnostics` lists them, each with a code, a place and a message. Log them while you try things:

```ts
for (const { code, range, message } of opened.diagnostics) console.warn(`${code} ${range.start.line}:${range.start.column} ${message}`);
```

Misspell `bind=done` as `bind=don`, and the page shows the input's fallback, *Done: 1*, while the list says `F030`: the input it names isn't declared. Only a request that isn't valid throws, such as a `now` that isn't an RFC 3339 time, or a `path` that isn't in `files`. [Processing documents](04-processing-documents.md) has the diagnostics in full.

Next: [how it fits together](03-how-it-fits-together.md), what runs where, and why.
