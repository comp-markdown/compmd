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

[Try it](https://compmd.hakanserce.com/playground/?doc=guide/quick-start) in the playground.

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
- `OpenDocument` read the document with what `request` holds (your packs, none yet), and gave its expanded tree and the components it uses.
- `createSandbox(sandbox)` made a strict frame for the page, from the frame documents the plugin serves under `compmd/`, and rendered the tree in it.
- The number input is a built-in component; when the reader changed it, the frame sent `set`, and nothing else can: the page's code runs in a frame with no network and no access to your page.

Next: [how it fits together](03-how-it-fits-together.md), what runs where, and why.
