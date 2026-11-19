# The host's code highlighter

**Status: proposed (2026-11-19), for review (CMD-267).** The sandbox takes its code highlighter from its host, in both
page modes, so a host whose editor highlights with its own engine (Lezer, say) shows the same colours in its rendered
pages, and languages Shiki doesn't ship. And where the default highlighter, Shiki, should run: in each frame, as today,
or in the host's page for all of them. CMD-262 builds what is decided here.

## Decisions (proposed)

1. **An option, `highlight`, on the sandbox,** beside `ownImages`: a `CodeHighlighter`, `@compmd/render`'s type,
   `(code, language) => lines | undefined | Promise<…>`, each token with its text and a colour for each scheme. Without
   it, Shiki highlights, as today.
2. **In host mode it is the page's highlighter,** given to `createPage` in place of the frames' Shiki.
3. **Frames ask the host's page (way B).** A frame's renderer asks its parent for a block's tokens with a message, and
   draws what comes back; the host's page highlights with the host's highlighter. Only data crosses into a frame, and
   the frames' code stays what the sandbox ships. A highlighter module the host names, loaded into the frames (way A),
   is not done: the host's code would run in strict frames beside untrusted packs' code, and a frame's code would be
   the host's choice, not the sandbox's (*Why not a module in the frames*, below).
4. **Two messages, in the sandbox protocol's next version (2; `@compmd/render`'s own frame protocol doesn't change):** `highlight {id, code, language}` from a frame to its
   parent, and `highlighted {id, lines?}` back, `lines` left out for a language the highlighter doesn't cover or when
   it fails. A parent answers only for a block of a tree, splice or part it sent that frame, as H answers `image` only
   for an image of what it sent: a frame can't make the host's page tokenize text of its own choosing. A permissive
   frame asks its own parent for its strict children, checking the same.
5. **A block keeps its text** until its tokens come, for a language left out, when its tokens aren't its text (the
   renderer checks), on a failure, and when the answer doesn't come within the sandbox's `timeout` (5 s by default),
   so `settled()` never waits longer than that for highlighting.
6. **The host's page answers a block at a time,** one a task, as the renderer's own queue asks for them (CMD-259):
   many frames asking at once make no long frame of their requests together.
7. **Shiki, the default, moves to the host's page too,** through the same messages: one Shiki for a page and all its
   frames, compiled once, instead of one in each frame. On the measurements below, a page of ten frames in WebKit
   blocks 0.4 s instead of 2.8 s, and its last frame's first block comes at 0.5 s instead of 3.3 s; forty frames, 0.7 s
   instead of 14 s. In Chromium with each frame in a process of its own, the frames' own Shikis keep the page's thread
   free but cost about 27 MB a frame. The frames then load no highlighter at all, so they no longer need
   `'wasm-unsafe-eval'` for it ([WebAssembly in the frames](webassembly-in-the-frames.md), which this reverses).
   *For the user to decide: [Shiki as the default](#shiki-as-the-default) has the comparison.*
8. **No batching** of several blocks in one message: a round trip costs a frame no more than its own Shiki does (one
   frame: 184 ms to its first block asking the page, 159 ms with its own, in WebKit), and the queue already orders the
   blocks.

## Today

- **The renderer** takes any `CodeHighlighter` (`createPage({ highlight })`), asks it for one block at a time, those on
  screen first, and draws the tokens into the block's own lines (`highlight-queue.ts`, CMD-251, CMD-259).
- **The sandbox** gives it Shiki, always:
  - in host mode, the frames' own Shiki loaded into the host's page, shared by its views (`framesHighlight`, CMD-256);
  - in every frame, the page frame of frame mode and each use frame of host mode whose slots hold code, a Shiki of
    the frame's own (`frame/runtime.ts`), with Oniguruma where the frame compiles WebAssembly outside Chromium
    (CMD-257, CMD-260).
- **A host can't choose:** only `createPage`, without the sandbox, takes another highlighter.

## The option, in use

A host whose editor highlights with Lezer gives the sandbox the same colours:

```ts
import { createSandbox } from "@compmd/sandbox";
import { lezerTokens } from "./my-editor/highlight.ts";

const sandbox = createSandbox({
  // Each token with its colour in each scheme, from the editor's two themes; `undefined` for a language it lacks.
  highlight: (code, language) => lezerTokens(code, language, { light: editorLight, dark: editorDark }),
});
```

- **Host mode:** the function runs in the host's page, for the page's own blocks.
- **Frame mode, and use frames in host mode:** the same function runs in the host's page, for each frame's blocks, as
  each frame asks.

## The messages

```text
P (or U)                         its parent (H, or a permissive U)
  highlight {id: 3, code: "let a = 1;", language: "ts"}  →
                                 is "let a = 1;" in a ts block of what I sent P?  no → highlighted {id: 3}
                                 yes → H's highlighter (one block a task)
  ←  highlighted {id: 3, lines: [[{text: "let ", color: {light: "#d73a49", dark: "#f97583"}}, …]]}
draws the tokens if they are the block's text; keeps the text otherwise
```

- **Which blocks a parent answers for:** each code block's text and language in the trees, splices and parts it sent
  the frame (a code renderer's block is its component's, not highlighted). Not a block's position: the same text in
  the same language gets the same tokens wherever it is.
- **`id`** is the frame's own count, so answers can come in any order.
- **A parent that isn't H** passes the request up, as its own, and the answer down.
- **The host's page and the frames it serves upgrade together,** as they did for the renderer's version 5: the
  version in `hello` and `ready` says which they speak, and the release notes say so.

## The threats

Against [the sandbox's threat model](running-components-in-the-sandbox.md):

- **What leaves a frame:** a block's text, which is the document's, from a tree the host's page sent: nothing the host
  didn't have. The check (decision 4) keeps a frame from sending text of its own to be tokenized.
- **What enters a frame:** tokens, which are data. The renderer draws a block's tokens only when their text is the
  block's, line by line, and sets only `color`, `font-style` and `text-decoration` on each, so a host's tokens can't
  add markup or change what the block says.
- **Grammars on untrusted text, in the host's realm:** a document's code runs through the host's highlighter in the
  host's page. Host mode does this already, with Shiki; with the option, it is the host's own code, as its editor
  runs it on the same text.
- **A frame's highlighting waits on the host's page:** a busy or stuck host's page leaves the frame's blocks as text,
  after the timeout at most (decision 5). A frame can't slow the host's page beyond its own blocks: one block a task.

### Why not a module in the frames

Way A, a highlighter module the host names by URL, loaded into the frames and checked by its hash as packs' modules
are, would run the host's code in strict frames, which run untrusted packs' code too. The host's code would share their
realm (a pack could patch what the highlighter calls), the frames' code would no longer be only what the sandbox ships
and checks, and each frame would load and compile the highlighter again, which is what decision 7 removes. Way B keeps
a frame's code fixed and costs a message a block.

## Shiki as the default

The question: with no `highlight` option, where does Shiki run for a page's frames?

- **In each frame (today).** Each frame loads Shiki, its engine and its grammars, and compiles them: in WebKit
  Oniguruma's WebAssembly (about 85 ms a frame) or, without WebAssembly, the JavaScript engine's regexes (about a
  second on a first TypeScript block); in Chromium the JavaScript engine.
- **In the host's page.** One Shiki for the page and all its frames, through the messages above. The frames load no
  highlighter.

### Three pages

- **A guide's chapter in frame mode:** one document, one page frame, a few code blocks. Either way costs about the
  same: about 0.2 s to the first block's tokens in WebKit, 0.1 s in Chromium.
- **A blog's front page showing ten posts in full,** each post its own document and so its own frame, five code blocks
  each: in WebKit, ten Shikis compile one after another in the page's process, and the page is blocked for 2.8 s while
  the last post waits 3.3 s for its first tokens; one Shiki in the host's page takes 0.4 s blocked, and every post's
  first tokens come by 0.5 s.
- **A review tool showing forty documents** (a pull request's changed files, each rendered as a document): in WebKit,
  14 s blocked and 16 s before the last document's first tokens, against 0.7 s for both with one Shiki in the host's
  page. In Chromium, 2.8 s blocked against 0.7 s when the frames share the page's process, and when each has its own
  process, the frames' work leaves the page's thread free (0.03 s blocked against 0.8 s) but costs 1.7 GB of heap
  against 0.6 GB.

### What each costs

From the measurements below, medians of three runs, five TypeScript blocks a frame. *Blocked* is the page's thread's
time in frames over 50 ms; *last first block* is when the slowest frame got its first tokens.

| Frames | Where Shiki runs | WebKit: blocked / last first block | Chromium, frames in the page's process: blocked / last first block / heap | Chromium, a process a frame: blocked / last first block / heap |
|---|---|---|---|---|
| 1 | each frame | 0 / 0.16 s | 0 / 0.13 s / 5 MB | 0 / 0.13 s / 6 MB |
| 1 | host's page | 0.17 s / 0.18 s | 0 / 0.09 s / 9 MB | 0.08 s / 0.13 s / 9 MB |
| 10 | each frame | 2.8 s / 3.3 s | 0.6 s / 1.0 s / 15 MB | 0 / 0.75 s / 119 MB |
| 10 | host's page | 0.4 s / 0.5 s | 0.2 s / 0.3 s / 13 MB | 0.2 s / 0.35 s / 48 MB |
| 40 | each frame | 14.0 s / 16.4 s | 2.8 s / 3.4 s / 45 MB | 0.03 s / 3.3 s / 1,693 MB |
| 40 | host's page | 0.7 s / 0.7 s | 0.7 s / 0.5 s / 35 MB | 0.8 s / 0.5 s / 616 MB |

| | Each frame (today) | The host's page |
|---|---|---|
| A frame's first block, one frame | 0.16 s (WebKit) | 0.18 s |
| A page of many frames | each compiles Shiki again: grows with the frames | compiled once: about flat |
| The page's thread | free only where the browser gives frames their own process (Chromium on the desktop); WebKit runs them in the page's | busy for the tokenizing, a block a task (decision 6) |
| Memory | a Shiki in each frame: about 27 MB a frame in its own process | one Shiki |
| What a frame needs | `'wasm-unsafe-eval'` for Oniguruma, its 230 KB chunk (gzipped) loaded in each | neither |
| What the host's page needs | nothing | Oniguruma where it compiles WebAssembly (`cspFor` allows it); without it, the JavaScript engine once in WebKit |

**What it means.** In WebKit (Safari, and the WebKit webviews of Tauri apps on macOS and iOS), frames run in the
page's process, so the frames' Shikis add up on the page's thread: one frame costs the same either way, and every
frame after it costs another compile today. In Chromium on the desktop, each cross-site frame has a process of its
own, so today's way keeps the page's thread free, at the cost of memory and of each frame's own wait; the host's page
then does the tokenizing on its own thread, a block a task. Moving the default is a large gain in WebKit from the second
frame on, a gain in memory everywhere, and costs Chromium's page thread up to a few hundred milliseconds spread over
the tokenizing; and it lets the frames drop `'wasm-unsafe-eval'`, which CMD-260 added to them for highlighting alone.

## The measurements

`spikes/highlight-perf`, `?framepage=N` (CMD-267): `N` frames live at once, each a document from another origin than
the page's (`127.0.0.1` against `localhost`), as the sandbox's frames are, each highlighting five TypeScript runs one
after another, as the renderer's queue asks for them. `where=frame`: each frame makes the sandbox's highlighter,
`shikiHighlight({ webAssembly: true })`, which picks Oniguruma in WebKit and the JavaScript engine in Chromium.
`where=host`: each frame asks the page over a `MessagePort`, and the page answers with one such highlighter, as soon as
each request comes (without decision 6's queue, which is why its longest frames are long: 0.4 s for ten frames in
WebKit, 0.6 s for forty, when the first blocks of all of them come at once). Playwright 1.64.1, headless, on an Apple M4.
Heap is Chromium's JavaScript heap after a collection, summed over the page's processes. WebKit has no such measure.

```sh
pnpm --dir spikes/highlight-perf measure 3 framepage-frame-1 framepage-frame-10 framepage-frame-40 framepage-host-1 framepage-host-10 framepage-host-40 --out=results-framepage.json
node spikes/highlight-perf/measure.ts 3 framepage-frame-1 framepage-frame-10 framepage-frame-40 framepage-host-1 framepage-host-10 framepage-host-40 --site-per-process --out=results-framepage-isolated.json
```

`results-framepage.json` has every run with Chromium's frames in the page's process (Playwright's headless Chromium
puts these frames there), and `results-framepage-isolated.json` with each in a process of its own
(`--site-per-process`, as desktop Chrome). WebKit's runs are in both, and agree.

**Caveats.** One machine, headless browsers, a local server, so loading Shiki's files is fast; over a network each
frame's 230 KB chunk and grammars would add to today's way and not to the host's page. Five blocks of TypeScript a
frame; other languages compile differently. The host's page here highlights nothing of its own.

## Open

- **Which colours a host's highlighter gives for the dark scheme,** when its editor has one theme: the option's docs
  show mapping two themes; a host with one gives the same colour twice.
- **A use frame's blocks in host mode** are rare (code blocks in a use's slots); they ask H the same way.
