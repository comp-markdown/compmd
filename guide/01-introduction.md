# 1. Introduction

CompMD is Markdown with components. A writer puts a box, a chart or an input in a document with a line of its own, and the document still reads well as text, on GitHub or in any Markdown viewer. Your application renders it as a page.

```md
---
read: 12
pages: 3480
md:
  inputs:
    goal: number = 20
---

::: grid {min=12rem gap=m}
::: box {padding=m border=subtle border-radius=m}
**{{md.doc.read}}** books read this year
::: /box
::: box {padding=m border=subtle border-radius=m}
**:value[3,480]{of=(md.number(md.doc.pages, 0))}** pages
::: /box
::: /grid

:: number-input[Goal: 20 books] {bind=goal}
```

This guide is for developers who put CompMD in an application: a notes app, a documentation site, a dashboard whose pages people write, an editor with a live preview. It shows how to render documents, give them components of your own and from packs, and write packs others can use.

## What CompMD gives your application

- **Documents your users write as text.** Markdown, which they know, with components, data, inputs and layout, which they can learn a line at a time. A document is a file they own: it diffs, it merges, it reads in any editor.
- **Pages you control.** The look of a page is yours: its stylesheet, its tokens, its components. The document says what is on the page; your application says how it looks.
- **Documents from anyone, safe to open.** Components written in CompMD run no code. Components written in JavaScript, React or Svelte run in a sandbox that sees only its own params, with no network, no storage and no way out of its box. A document from a stranger can't reach your page, your session or your data.
- **Diagnostics with fixes.** Every mistake has a code, a place and, where one is clear, the edit that fixes it; an editor can show them as the writer types, with hints at the caret.

## The packages

| Package | What it does |
|---|---|
| [`@compmd/processor`](../wasm/README.md) | Reads a document, with everything it needs, and gives its trees, its diagnostics and its components. The processor in Rust, as WebAssembly. |
| [`@compmd/render`](../render/README.md) | Renders the expanded tree as a live page, and updates it in place as the document changes. |
| [`@compmd/sandbox`](../sandbox/README.md) | Runs components at the trust their pack is given: in frames with no network, or your own in your page. |
| [`@compmd/vite`](../vite/README.md) | Reads `compmd.config.json`, gives the processor your packs, and serves the sandbox and the packs' files from your app. |
| [`@compmd/create-pack`](../create-pack/README.md) | Writes a new pack: a component as one built module, its signature, and a test page. |

A document goes through them like this:

```text
  notes.comp.md ──► @compmd/processor ──► expanded tree ──► @compmd/sandbox ──► the page
                         ▲                 + components       └ @compmd/render
                         │
        @compmd/vite: your packs, compmd.config.json, the sandbox's files
```

The processor does no I/O of its own: your application gives it the document, the files it reads, its packs and its data, and gets back everything the page needs. That's what lets the same processor run in a browser, in a worker, in a build, or natively in Rust.

## How to read this guide

[Part I](README.md#contents) gets you from nothing to a page in a few minutes, then explains what runs where. The parts after it each take one side of building with CompMD, and can be read in any order. Each chapter's code is a working example in [`examples/`](examples), and a document with a **Try it** link opens in the playground, where you can change it and watch the page.

Next: [the quick start](02-quick-start.md).
