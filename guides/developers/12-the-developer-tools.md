# 12. The Developer Tools

`@compmd/devtools` shows what a document becomes and what its sandbox does: its diagnostics, its components, its frames, its data, the messages between its windows. This chapter puts its inspector under an app's page in development, embeds its editor, and goes through what each tab tells you and how far to believe it.

## Two ways to embed it

| | The editor | The inspector |
| --- | --- | --- |
| What it shows | a document's text and its page, side by side, and the tabs | the tabs, for a page your app already renders |
| Its page | its own, in a sandbox it opens | yours: it observes your view |
| You give it | `sandbox` (as `createSandbox` takes it) and `document` | `view` and `document` |
| For | a tool to write documents in, a preview, a playground | seeing what your app's documents do, in development |

Both are one function, `mountDevtools(target, options)`, whose options say which. The tools are written in Svelte and compiled into the package, so a page in any framework embeds them, with no Svelte of its own. They render in a shadow root of `target`, so your page's styles can't reach in, and theirs can't leak out. They fill the element you give them.

```sh
npm install @compmd/devtools
```

## The inspector, in development

[`@compmd/vite`'s example](../../vite/README.md#the-example) opens the inspector under its page on the dev server, on the view it renders. Its `main.ts` does it in a function it calls only in the dev server's mode, so a build leaves the tools out:

```ts
/** In development, the inspector on the page's view, in a panel under it: loaded only then, so a build leaves it out. */
async function inspect() {
  const { mountDevtools } = await import("@compmd/devtools");
  const panel = document.getElementById("inspector")!;
  panel.style.height = "60vh";
  return mountDevtools(panel, { view, document: { request: requestOf() } });
}
// The dev server's mode: `DEV` follows `NODE_ENV`, which a build run by a test runner may set to `test`.
const inspector = import.meta.env.MODE === "development" ? await inspect() : undefined;
```

`view` is what `createSandbox(…).open(…)` gave, and `document` the request the app processed, which the inspector processes again in a worker of its own for its tabs. When the app processes the document again, it tells the inspector:

```ts
view.on("set", ({ input, value }) => {
  inputs = { ...inputs, [input]: value };
  document_.free();
  const processed = requestOf();
  document_ = new OpenDocument(processed);
  view.render(document_.expanded);
  inspector?.open({ request: processed });
});
```

The inspector observes the view from when it mounts, however long the view has been open: the page and each frame report what is live first. `destroy()` removes it and stops observing.

The tools process documents in workers, which are files of their own beside the package's code, with the processor's WebAssembly. Vite builds a package's workers only when it doesn't pre-bundle the package, so list it in `optimizeDeps.exclude`, as the example's `vite.config.ts` does:

```ts
import { compmd } from "@compmd/vite";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [compmd()],
  build: { target: "es2024" },
  // The inspector's workers are files of their own, which Vite builds only in a package it doesn't pre-bundle.
  optimizeDeps: { exclude: ["@compmd/devtools"] },
});
```

## Embedding the editor

The editor is the playground's: the text with the hint at the caret and the diagnostics under it, the page beside it, updated as you type. Give it where your frames are and a document:

```ts
import { mountDevtools } from "@compmd/devtools";
import { request, sandbox } from "virtual:compmd";
import text from "./notes.comp.md?raw";

const tools = mountDevtools(document.getElementById("tools") as HTMLElement, {
  sandbox,
  document: { request: { ...request, path: "notes.comp.md", files: { "notes.comp.md": text }, now: new Date().toISOString() } },
});
```

| Option | What it is |
| --- | --- |
| `sandbox` | the sandbox the page renders in, as `createSandbox` takes it: `virtual:compmd`'s `sandbox`, with `page: "host"` for host mode; frame mode by default |
| `document` | the document to open: `request`, a processor's request with its text in `files`, and `files`, what it reads, shown read-only beside its text (`{ name, text }`) |
| `registries` | where to fetch the packs a document imports that its request lacks, as `virtual:compmd`'s `registry` gives them ([chapter 9](09-packs-from-npm.md)); without it, those are F026 |
| `stylesheets` | your stylesheets for the page, served from the frames' origin |
| `select` | a passage of the text to select when the document opens, a text fragment's `text=` value |
| `shareLink` | given the same value for the selected text, the link that opens it: the tools then offer to copy a link to the selection |

The handle has `open(document)`, which opens another document in a new page; `styles(urls)`; `state`, with the diagnostics, problems and timings; and `destroy()`. Each key press is an edit, processed in a worker that keeps the document open, so only the blocks it touches are processed again, and the page follows as splices.

## Reading the tabs

In order: the page first, then why something is wrong, then what the document is made of, then the raw material. The inspector has every tab but the page, which is yours.

| Tab | What it shows |
| --- | --- |
| **Page** | the document rendered (the editor only), kept mounted under the other tabs |
| **Console** | what went wrong, newest last: each change's diagnostics, packs that couldn't be fetched, and from the sandbox each component that shows its fallback with why (W118, W121, `not-here`), each `set` it refused, each frame it stopped |
| **Components** | those the document defines and those it uses, and those it could use |
| **Sandbox** | the windows as a tree, from the host's page down to the frames its frames make: each with its level, its network, what it renders and its last render's time, the components mounted in it with how long they took to mount and update, and its fallbacks |
| **Data** | what the frontmatter gives the render: its fields, each data import with what was loaded, the inputs, the settings |
| **Plain Markdown** | what a plain CommonMark and GFM renderer shows, in a frame without scripts |
| **Messages** | the sandbox's protocol on a timeline, a lane per frame, each message opening as the trees do |
| **Parse tree**, **Expanded tree** | the processor's trees as JSON that folds |

In the editor, a bar over the tabs lists the document's inputs, each with a control by its type; changing one there opens the document again with it. In the inspector, the inputs are your page's, and its Data tab shows them.

A component's time is its `mount`'s and its updates' calls. A React component's mount schedules its render, so most of its time is in the frame's own render time.

## The observer behind them

The Console, Sandbox and Messages tabs read the sandbox's observer, which any developer tool can use. Only your page turns it on, with the view:

```ts
const observation = view.observe((events) => {
  show(observation.events);
});
observation.dropped;   // what was dropped, by source
observation.stop();    // the sandbox observes while an observation listens
```

`events` holds what the host's page saw and what its frames reported, in the order it came: `frame` and `gone` as frames come and go (with why: `removed`, `hung`, `version`), `message` for each message on a port (`down` or `up`, pings left out), `mounted`, `updated` and `unmounted` with their times, `fallback` with its reason, `refused` for a `set` the parent dropped, and `dropped`. Each has its time, `at`. Frames are ids that are paths: `"2/1"` is the first frame of the second.

Nothing a document, a pack or a frame sends turns it on. Off, nothing is copied. On, it keeps the last 2,000 events of each source, your page and each frame, so a frame that floods it pushes out only its own; long values are kept by their size (`{ $text: length, start }`). It changes nothing the sandbox does, and sends nothing anywhere.

## What "reported by" means

Your page sees its own frames and the messages on their ports at first hand. What happens inside a frame (the frames a permissive frame makes, the components mounted in a page frame, their times) only that frame can see, so it reports it, once its parent asks. Such an event has `by`, the frame whose report it is, and the tabs mark it **reported by** that frame.

Code of that frame's level shares the frame's window with the code that reports, and could change what it says: a component could report that it rendered in 2 ms with no network when it did something else. The tools can't tell, so they mark it, show everything a frame sends as text, never as markup or a link, and nothing acts on it: which frame gets `host`, which `set` passes and which frame is stopped come from the sandbox's own checks, never from a report. A frame also learns it is observed when its parent asks, so code of its level can behave differently while the tools are open. The tools show what a frame does, not what it would do unobserved ([the design's accepted risk](../../docs/devtools.md#the-accepted-risk)).

A lie gains little: a frame is never less trusted than the frames it reports on, so code that can falsify a report controls what it reports on anyway.

Next: [your first pack](13-your-first-pack.md), a component of your own for any app's documents.
