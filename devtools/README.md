# @compmd/devtools

The CompMD Developer Tools as a part any page embeds ([the design](../docs/devtools.md)): a document's text on the left, its page on the right, updated as you type, with the views of what the processor makes of it and of what its sandbox does. The [playground](../editor/README.md) is a page that embeds it.

**Status: in development;** not published yet. This is the design's first increment, the editor; the inspector, attached to a view a host app already renders, comes next.

```ts
import { mountDevtools } from "@compmd/devtools";

const tools = mountDevtools(element, {
  sandbox: { base: "/sandbox/", modules },             // as `createSandbox` takes it (@compmd/sandbox)
  document: { request, files },                       // a processor's request, and the files shown beside its text
  registries: { registries: { "*": "https://registry.npmjs.org/" }, fetch: "direct" },   // for packs the request lacks
  stylesheets: ["/sandbox/docs.css"],                 // the host's stylesheets for the page
  shareLink: (select) => `${location.pathname}?select=${select}`,   // offers to copy a link to the selected text
});
tools.open(other);                                    // another document, in a new page
tools.styles(urls);
tools.state;                                          // diagnostics, problems, timings
tools.destroy();
```

It is written in Svelte and compiled into the package, so a page in any framework embeds it with this function, and needs no Svelte of its own. It renders in a shadow root, with its stylesheet, so the page's styles can't reach in and its own can't leak out.

**Its workers** are files of their own beside it, which it starts with `new Worker(new URL("./worker.js", import.meta.url))`; they import `@compmd/processor`, whose WebAssembly is a file too. A bundler that builds workers (Vite, webpack 5) builds them with the host's page. Vite does so only for a package it doesn't pre-bundle: list `@compmd/devtools` in `optimizeDeps.exclude`.

## How it works

- **Processing runs in a worker,** never in the text area's input handler, so each key press is painted first and the page follows it. The worker keeps the document open with [`@compmd/processor`](../wasm/README.md)'s `OpenDocumentJson`, so an edit inside a block processes that block, not the document. Its nodes carry origins, which the renderer's keys come from.
- **Each input is an edit.** The selection before the input is the range it replaced, so a cut or a paste is placed where the user made it. Backspace, Delete and undo change text outside the selection, so the range then also covers what changed (`src/edit.ts`). Edits made while the worker is busy go to it together, in order (`src/session.ts`).
- **Splices, not trees, cross to the frame.** The worker gives a splice per edit, as the processor's JSON text, and the editor passes it on unread with the text and its diagnostics. Only the page's frame holds the expanded tree, as the renderer's blocks.
- **The page renders in [`@compmd/sandbox`](../sandbox/README.md)**, as the host's options say (its strict frame, frame mode, by default), one view per document. The renderer applies each splice and renders only the blocks it replaced; its keys follow each splice's edit, so components keep their state through edits around them.
- **The hint line**, under the text and above the diagnostics, says what is at the caret and what it takes, from the processor's [positional hints](../processor/docs/positional-hints.md), with the diagnostic at the caret first and its fix. A param's value and a type name offer their values as buttons, which replace what is written, as a fix does. Each message to the worker carries the caret, so each answer brings the hint at it; a caret that moves without an edit asks for the hint alone, once the worker is free (`src/session.ts`, `src/hint.ts`).
- **Diagnostics** are listed under the text, each with its code, place and message; clicking its place selects its range in the text. A diagnostic that suggests a name or a token has a button that makes its fix, as an edit.
- **Inputs:** a bar over the tabs lists the document's inputs, each with a control by its type. Changing one there, or through the page (an input component's `set`), opens the document again with the new value.
- **Links to the text:** a link in the page to a text fragment (`#:~:text=`, spec §2.3) selects that passage in the text, or says it isn't there any more (`src/fragment.ts`). With text selected, and `shareLink` given, "Copy link to selection" copies the link it makes. A link to another site opens in a new tab: only the reader's own clicks open one, since a component may not navigate (spec §15).
- **Packs from a registry:** with `registries`, the worker asks which packs the document imports that its request lacks, once typing pauses, and resolves them with `@compmd/registry`'s `resolvePacks`, kept in Cache Storage. Until they arrive the document renders with what it has; when they do, it opens again, and their built modules reach the sandbox as checked code (`src/processing.ts`, `src/worker.ts`).
- **The look** (`src/devtools.css`): the text and the page as two sheets on a desk, light and dark, in the system's fonts. The handle between them drags, or moves with the arrow keys; Home and End fold a side away, and the browser remembers where it was (`src/split.ts`). On a narrow screen the sheets stack.

## The tabs

In the order of decision 8: the page first, then why something is wrong, then what the document is made of, then the raw material.

- **Page:** the document rendered, which stays mounted under the other tabs, with its state.
- **Console:** what went wrong, newest last: each change's diagnostics when they change, what the editor couldn't do (a pack it couldn't fetch), and, from the sandbox's observer, each component that shows its fallback with why (W118, W121, `not-here`), each `set` the sandbox refused, each frame it stopped (`src/console.ts`).
- **Components:** those the document defines and those it uses, each line selecting it in the text, and those it could use.
- **Sandbox:** the page's windows as a tree, from the host's page down to the frames its frames make, each with its level, its network, what it renders and its last render's time, the components mounted in it with how long they took to mount and update, and its fallbacks (`src/frames.ts`).
- **Data:** what the frontmatter gives the render: its fields, each data import with what was loaded, the inputs, the settings.
- **Plain Markdown:** what a plain CommonMark and GFM renderer shows (spec §14), in a frame without scripts.
- **Messages:** the sandbox's protocol on a timeline, a lane per frame, each message's copy opening as the trees do (`src/messages.ts`).
- **Parse tree** and **Expanded tree** (spec §12): JSON that folds, two levels open. A second worker makes these views from the current text while one of their tabs is open (`src/views.ts`).

The Console, Sandbox and Messages tabs read [the sandbox's observer](../sandbox/README.md#observing-a-view), which the tools start with each page: frames report what happens in them, so the tabs mark what a frame reported as that frame's, since code of its level could have changed it ([the accepted risk](../docs/devtools.md#the-accepted-risk)), and show everything a frame sends as text, never as markup or a link (O1). A component's time is its mount's and its updates' calls: a React component's mount schedules its render, so the frame's own render time includes the rest.

## Testing a host's documents

`@compmd/devtools/testing` has what a host's tests use to check its documents as the tools process them: `Processing`, which the tools' worker runs; `fixOf`, a diagnostic's fix as an edit; and `parseFragment` and `findPassage`, the passage a text-fragment link names. The playground's tests use them on its documents.

## Commands

`@compmd/processor`, `@compmd/render`, `@compmd/registry` and `@compmd/sandbox` must be built first (`pnpm -r build` from the root does them all).

```sh
pnpm build   # dist/ with rolldown: the Svelte compiled in, the workers beside the code, the other packages left to the host
pnpm lint    # eslint, tsc, and svelte-check for the .svelte files
pnpm test    # edits, the session, the editor's loop, the processing, the views, and the observer tabs, in happy-dom
```

The build is rolldown's, not Vite's: Vite's library build marks a worker's URL to be ignored, which a host's bundler then never builds.

## License

[MIT](LICENSE). Copyright © 2026 Hakan Serce.
