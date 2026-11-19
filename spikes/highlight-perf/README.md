# Spike: code highlighting's cost in WebKit (CMD-252)

**Status: done (2026-11-19).** A host that shows many code blocks (a diff viewer: hundreds of short runs, each a view of its own) found that in WebKit highlighting blocks the page for seconds, while Chromium is smooth. This spike measures it on a page of our own and tries each way to lower it. Nothing here changes the packages; what to carry over is proposed at the end, as tasks.

## Decisions it supports

1. **One Shiki per page, shared by its views.** Today each view makes its own (`framesHighlight` calls `shikiHighlight()` per view, and Shiki warns about it), so each run pays for its grammar again: that alone is most of the skim's jank in WebKit (5.9 s blocked, against 0.9 s shared).
2. **Oniguruma, as WebAssembly, where the page allows WebAssembly.** WebKit's cost is the JavaScript engine compiling TypeScript's regexes the first time they run: a 1.1 s frame on opening, and later long frames as new rules first match. With Oniguruma, shared, the first tokenizing takes 30 ms, and the page meets the host's targets: 83 ms blocked on opening, its longest frame 133 ms, 10 ms blocked over the skim. In Chromium it is as fast as the JavaScript engine once shared.
3. **Tokenizing in slices, a block per task, those on screen first,** as a guard: with Oniguruma it takes opening's last 83 ms to 0, and it bounds a frame to one block however many blocks render at once.
4. **Not carried over:** grammars prepared ahead (no gain in WebKit, which still compiles each regex on first use), a cache of tokens (nothing left to save once Shiki is shared: a run tokenizes in about 1 ms), and a worker (as smooth as Oniguruma on the page's thread, but a run waits 1.1 s on opening with the JavaScript engine, and it needs `worker-src` where Oniguruma needs `wasm-unsafe-eval`).
5. **A finding apart from highlighting:** every `createPage` adopts a stylesheet of its own, with the base rules, so a document with hundreds of mounted pages restyles against hundreds of sheets: 43 s blocked in WebKit over the skim with highlighting off, 304 s with it. The spike doesn't fix it: CMD-253 does, with one sheet for a document's pages, and measured what each open costs in a large host document ([below](#a-large-host-document-100-pages-opened-and-closed-cmd-253)).

## The test page

`index.html` and `src/`: 440 runs of TypeScript, 4 to 30 lines each, cut from this repository's own sources, the same each time (`runs.ts`). Each run is a page of its own (`@compmd/render`'s `createPage`, a document of one fenced block, with its origin), mounted when it comes within a screen of view and destroyed once it leaves, as a virtual list's items are (`?keep=1` keeps them mounted). The highlighter is chosen by the URL's query (`highlighters.ts`):

| Choice | Values |
|---|---|
| `shiki` | `per-view` (a Shiki for each page, as today), `shared` (one for the page) |
| `engine` | `js` (the JavaScript engine, forgiving, as the sandbox uses), `precompiled` (Shiki's precompiled languages, with its raw JavaScript engine), `oniguruma` (WebAssembly, inlined) |
| `where` | `main` (the page's thread), `worker` (one worker for the page) |
| `schedule` | `eager` (each block as it renders, as today), `sliced` (one block per task, those on screen first) |
| `cache` | `1`: tokens kept by their text, for a run mounted again |

Each variant draws the tokens into the renderer's line elements as the sandbox's highlighter does (`draw.ts`), in GitHub's light and dark themes.

## How it is measured

`measure.ts` opens the page in Playwright's WebKit and Chromium (1.64.1, headless, 1280×800), on this machine (Apple M4), served locally, and runs three phases:

- **Opening:** from the first runs mounting (6 on screen and a screen ahead) until they are highlighted.
- **Skim:** to the end, a screen and a half every 100 ms, then until every mounted run is highlighted.
- **Revisit:** every page destroyed, back to the top, and the skim again, as a reader jumping back.

**Blocked** is the time in frames over 50 ms, past 50 ms, from the gaps between animation frames (WebKit has no long-task API); **longest** is the longest frame; **wait** is a run's median time from mounting to highlighted. Each variant ran three times; the tables give the median (the three openings were within 10% of each other in every variant). `results.json` has every run; `results-first-pass.json` the first pass, once each, with the variants left out of the repeat.

```sh
pnpm --dir spikes/highlight-perf measure                 # every variant, once
node spikes/highlight-perf/measure.ts 3 today shared     # some variants, three times each (after `vite build`)
```

## Results

### WebKit

| Variant | Opening: blocked / longest | Skim: blocked / longest | Revisit: blocked | Wait on opening | First tokenizing | A run's tokenizing |
|---|---|---|---|---|---|---|
| no highlighting (first pass) | 0 / 23 ms | 0 / 30 ms | 0 | — | — | — |
| **today** | 1139 / 1071 ms | 5852 / 719 ms | 4577 ms | 1114 ms | 1030 ms | 14 ms |
| shared | 1171 / 1221 ms | 879 / 695 ms | 0 | 1231 ms | 616 ms | 1 ms |
| shared + precompiled | 1484 / 1534 ms | 932 / 740 ms | 8 ms | 1549 ms | 372 ms | 1 ms |
| shared + sliced | 1060 / 642 ms | 734 / 462 ms | 0 | 1093 ms | 615 ms | 1 ms |
| shared + sliced + precompiled | 1160 / 914 ms | 686 / 463 ms | 0 | 1217 ms | 313 ms | 1 ms |
| **shared + oniguruma** | 83 / 133 ms | 10 / 57 ms | 0 | 157 ms | 30 ms | 1 ms |
| **shared + sliced + oniguruma + cache** | 0 / 43 ms | 0 / 46 ms | 0 | 127 ms | 27 ms | 1 ms |
| worker | 0 / 19 ms | 0 / 43 ms | 0 | 1104 ms | 630 ms | 1 ms |
| worker + oniguruma | 0 / 18 ms | 0 / 50 ms | 0 | 127 ms | 31 ms | 2 ms |
| worker + sliced + precompiled + cache | 0 / 18 ms | 0 / 46 ms | 0 | 1184 ms | 304 ms | 1 ms |

### Chromium

| Variant | Opening: blocked / longest | Skim: blocked / longest | Revisit: blocked | Wait on opening | First tokenizing | A run's tokenizing |
|---|---|---|---|---|---|---|
| today | 50 / 83 ms | 217 / 100 ms | 150 ms | 117 ms | 72 ms | 12.1 ms |
| shared | 33 / 83 ms | 17 / 67 ms | 0 | 103 ms | 34 ms | 1.4 ms |
| shared + oniguruma | 50 / 100 ms | 0 / 17 ms | 0 | 143 ms | 26 ms | 1.6 ms |
| shared + sliced + oniguruma + cache | 0 / 33 ms | 0 / 17 ms | 0 | 114 ms | 26 ms | 1.8 ms |
| worker | 0 / 17 ms | 0 / 17 ms | 0 | 85 ms | 35 ms | 1.5 ms |
| worker + oniguruma | 0 / 17 ms | 0 / 17 ms | 0 | 253 ms | 57 ms | 2 ms |

### Every page kept mounted (`?keep=1`, first runs, once)

| | WebKit: skim blocked / longest | Chromium: skim blocked / longest |
|---|---|---|
| no highlighting | 43 459 / 1679 ms | 17 300 / 900 ms |
| shared | 304 346 / 11 708 ms | 24 166 / 1300 ms |

### The sandbox's own highlighter, per view and shared (CMD-256)

`?sandbox=per-view` and `?sandbox=shared` give each page `@compmd/sandbox`'s `shikiHighlight()` as its `highlight`, drawn by the renderer (CMD-251): a highlighter for each page, as a host-mode view made one before CMD-256, or one for all, as `framesHighlight` shares it since. Three runs each, medians; `results-sandbox.json` has every run. Its tokenizing isn't counted here, so the table has no such column.

| | Opening: blocked / longest | Skim: blocked / longest | Revisit: blocked | Wait on opening |
|---|---|---|---|---|
| WebKit, per view | 1147 / 1077 ms | 4174 / 735 ms | 3529 ms | 1285 ms |
| WebKit, shared | 1142 / 1192 ms | 772 / 679 ms | 0 | 1227 ms |
| Chromium, per view | 50 / 83 ms | 284 / 133 ms | 150 ms | 169 ms |
| Chromium, shared | 33 / 83 ms | 67 / 117 ms | 0 | 106 ms |

The same as the spike's *today* and *shared*: one highlighter takes WebKit's skim from 4.2 s to 0.8 s blocked and its revisit to 0; the opening's long first frame is the JavaScript engine's compile, which CMD-257 is for.

### The sandbox's shared highlighter with Oniguruma (CMD-257)

`?sandbox=shared&wasm=1`: the same, with `shikiHighlight({ webAssembly: true })`, as a host-mode page that allows WebAssembly has it since CMD-257; this page has no policy, so it compiles. Three runs each; `results-sandbox-wasm.json` has every run. WebKit's runs varied more than before (one JavaScript run took twice the others, and the Oniguruma openings were 103, 253 and 265 ms), so each cell is the median and the range.

| | Opening: blocked | Skim: blocked | Revisit: blocked | Wait on opening |
|---|---|---|---|---|
| WebKit, JavaScript engine | 1175 ms (1144–2417) | 921 ms (783–2072) | 72 ms (0–115) | 1261 ms |
| WebKit, Oniguruma | 253 ms (103–265) | 151 ms (0–576) | 94 ms (0–105) | 375 ms |
| Chromium, JavaScript engine | 33 ms | 67 ms | 0 | 111 ms |
| Chromium, Oniguruma | 67 ms (50–200) | 0 (0–17) | 0 | 177 ms |

- **In WebKit Oniguruma takes the opening from about 1.2 s blocked to about 0.25 s,** and the longest frame from about 1.2 s to 0.3 s; the skim from 0.9 s to 0.15 s.
- **It is short of the spike's *shared + oniguruma*** (83 ms on opening, 10 ms over the skim): the sandbox's highlighter has two Shikis (Markdown's and the other languages'), each made with the first block, and loads its grammars as blocks need them; and this run was noisier. Slicing (CMD-259) is for what is left.
- **The JavaScript engine's tokens in WebKit** were wrong for one block in the browser test (`test/highlight-policy.test.ts`): its light theme's colours disagreed with its dark theme's, and its tokens with Oniguruma's and with Chromium's. Oniguruma's were right. Not looked into here: CMD-261.

### A block at a time, those on screen first (CMD-259)

The same two variants, with the renderer asking for one block's tokens at a time since CMD-259: the next a task after the last is drawn, blocks on screen first. Three runs each, medians and ranges; `results-sandbox-queued.json` has every run.

| | Opening: blocked / longest | Skim: blocked | Revisit: blocked | Wait on opening |
|---|---|---|---|---|
| WebKit, JavaScript engine | 1698 ms (1621–2437) / 1748 ms | 901 ms (787–1357) | 0 | 1809 ms |
| WebKit, Oniguruma | 84 ms (77–110) / 134 ms | 8 ms (0–13) | 0 | 158 ms |
| Chromium, JavaScript engine | 33 ms | 83 ms | 0 | 111 ms |
| Chromium, Oniguruma | 50 ms | 0 | 0 | 155 ms |

- **With Oniguruma, WebKit now meets the host's targets** (no frame over 200 ms on opening, well under a second over the skim): 84 ms blocked on opening, its longest frame 134 ms, 8 ms over the skim, against 253 ms and 151 ms before. That is the spike's best (*shared + oniguruma*: 83 ms and 10 ms).
- **With the JavaScript engine it is no better, and its opening looks worse** (1.7 s against 1.2 s before, in runs that varied a lot): a block at a time can't split one block's first compile of its regexes, and the blocks' compiles no longer share a frame. A page that refuses WebAssembly keeps that cost; the frames do until CMD-260.

### A large host document: 100 pages opened and closed (CMD-253)

The page above has a small host document, so it missed what each open costs a large one. `?churn=1` adds 20,000 host elements (rows a host's own rules style), then opens 100 pages, one an animation frame, eight live at a time, destroying the oldest, without highlighting: `churn` with a code block each, `churn-styled` with the code in a box with a `width` of its own. It counts what makes the browser style the whole document again: the times the document's sheets are set (`adoptedStyleSheets`), the styles read (`getComputedStyle`) and the sheets written (`replaceSync`). **An open** is from `createPage` to the next animation frame, so 16 ms is one frame. Three runs each, medians; `results-churn-before.json` and `results-churn-after.json` have every run.

| | Sheets set | Styles read | Sheets written | Blocked | An open: first ten / last ten / longest |
|---|---|---|---|---|---|
| WebKit, `churn`, before | 192 | 200 | 100 | 2609 ms | 31 / 71 / 94 ms |
| WebKit, `churn`, after | 1 | 0 | 1 | 0 | 16 / 17 / 18 ms |
| WebKit, `churn-styled`, before | 192 | 200 | 100 | 2619 ms | 38 / 74 / 93 ms |
| WebKit, `churn-styled`, after | 1 | 0 | 193 | 0 | 36 / 38 / 45 ms |
| Chromium, both, before and after | as WebKit's | | | 0 | 16 / 16 / 17 ms |

- **Before, every open set the document's sheets and read a style at once,** which styled all 20,000 elements inside the open: 92 of 100 frames were long in WebKit, and an open grew from 31 ms to 71 ms over the run (why it grows wasn't looked into). The two reads a page are the page's own: one when it is made, one at its render. A sandbox's view in host mode read twice more, at open and at each render.
- **After, a page of text and code changes nothing in the document's styles:** one sheet set for the first page, no style read, and each open within a frame.
- **A page with styled blocks still writes the sheet** when it renders and when it is destroyed (193 writes), and in WebKit an open took about two frames (38 ms) in this run, without a long frame. The writes aren't what that costs ([below](#writing-a-blocks-rules-cmd-254)).
- **Chromium doesn't show any of it** at this size, before or after.
- **A real host's document is heavier** than 20,000 plain rows: the host that reported it saw about 1 s for each of these restyles, where this page has 70 ms.

### Writing a block's rules (CMD-254)

Whether a styled block's rules should be inserted and deleted one by one (`insertRule`, `deleteRule`) rather than the document's sheet written whole (`replaceSync`, as `Stylesheet.flush` does). Two ways, in the same large host document, three runs each, medians; `results-sheet.json` and `results-sheet-styled.json` have every run.

- **The writes alone** (`sheet-none`, `sheet-replace`, `sheet-insert`): one adopted sheet with the base rules, 100 blocks added and removed, eight live, one a frame, each with a rule of its own and one in a container query, written one way or the other, or not at all.
- **The renderer's pages** (`churn-styled`), against the same with the sheet writes counted but not made (`churn-styled-nowrite`), so what is left is the rest of an open.

| WebKit | Blocked | An open: first ten / last ten / longest |
|---|---|---|
| No rules written | 0 | 16 / 16 / 18 ms |
| The whole sheet each time | 0 | 17 / 18 / 20 ms |
| Rule by rule | 0 | 17 / 19 / 22 ms |
| The renderer's styled pages | 0 | 20 / 20 / 24 ms |
| The same, the writes not made | 0 | 20 / 26 / 30 ms |

Chromium: 16 ms an open in every one.

- **Writing the sheet costs little, either way:** a frame and a millisecond or two with sixteen rules live, and rule by rule is no cheaper than the whole sheet.
- **What a styled page's open costs beyond a code block's is its own boxes,** not the writes: without them the open is no faster.
- **So the renderer keeps writing the sheet whole.** Hundreds of styled pages live at once, with hundreds of blocks' rules in the sheet, weren't measured; that is what could change it.
- This run's styled opens took 20 ms where the CMD-253 run above measured 38 ms, on the same page; why the two runs differ wasn't looked into. The comparisons are within a run.

```sh
pnpm --dir spikes/highlight-perf measure 3 churn churn-styled --out=results-churn.json
pnpm --dir spikes/highlight-perf measure 3 sheet-none sheet-replace sheet-insert --out=results-sheet.json
pnpm --dir spikes/highlight-perf measure 3 churn churn-styled churn-styled-nowrite --out=results-sheet-styled.json
pnpm --dir spikes/highlight-perf measure 3 sandbox-per-view sandbox-shared --out=results-sandbox.json   # after building @compmd/sandbox
```

## What the numbers say

- **WebKit's extra cost is compiling, not tokenizing.** Once a Shiki has run its grammar, a run tokenizes in 1 ms in both browsers. What WebKit pays is the JavaScript engine's regexes on their first use: 616 ms for the first run (34 ms in Chromium), and later long frames as rules not met yet first match (the shared variant's 0.9 s over the skim). Precompiling the grammars converts the regexes ahead, but WebKit still compiles each on its first run, so it doesn't help (372 ms first, the same frames).
- **Per view, that cost comes back.** Today every view makes a Shiki of its own, and each pays part of it again (a run's tokenizing is 14 ms, not 1 ms): 5.9 s blocked over the skim and 4.6 s again on the revisit.
- **Oniguruma doesn't have it.** Its first tokenizing is 30 ms in WebKit, and nothing after it is long. Per view it is slow (each view starts its own WebAssembly: 50 ms a run in WebKit, 40 ms in Chromium, in the first pass); shared, it is as fast as the JavaScript engine in both browsers.
- **Slicing bounds a frame to one block,** but can't split the first block's compile: with the JavaScript engine the opening frame drops from 1.2 s to 0.6 s, no further. With Oniguruma there is nothing long to split, and it takes the opening's 83 ms to 0.
- **A worker keeps the page smooth whatever the engine, but not the wait:** with the JavaScript engine a run is highlighted 1.1 s after mounting, since the same compile happens in the worker. With Oniguruma in the worker, both are fine.
- **A cache saves nothing here:** once Shiki is shared a run tokenizes in 1 ms, so the revisit is already 0 without it. It may matter for long blocks seen again; not measured.
- **The host's own numbers** (WebKit, 10–11 s blocked over a skim, 24 ms a run, 1.3 s on opening) are of the shape of today's variant here; its runs are longer than these. Its targets, well under 1 s over the skim and no frame over 200 ms on opening, are met by **shared + Oniguruma** and by the variants with a worker and Oniguruma, and by nothing with the JavaScript engine on the page's thread.

## What it means for the packages

- **The strict frames' policy refuses WebAssembly and workers** (`build/policy.ts`: no `wasm-unsafe-eval`, `worker-src 'none'`). Oniguruma in the frames needs `wasm-unsafe-eval` in their `script-src`, a security decision of its own: what a compromised or malicious frame could do with WebAssembly that it can't with JavaScript, in a frame that already runs untrusted code. That is a design document before code.
- **In host mode** the page is the host's: Oniguruma needs the host page to allow `wasm-unsafe-eval`, which `@compmd/vite`'s `cspFor` would add.
- **A page that refuses WebAssembly** keeps the JavaScript engine, shared, which is the most of today's gain without Oniguruma.
- **Size:** the WebAssembly is 622 KB (232 KB gzipped), loaded the first time a page has a code block, against the JavaScript engine's 56 KB. Served locally here; a slow network would move the first highlight later, not block the page.

## Caveats

- One machine (Apple M4), headless browsers, a local server; the host's WebKit is the app's, not Playwright's build.
- TypeScript only: other grammars compile differently, and Markdown with CompMD's grammar injected is the other one the sandbox highlights often.
- Each run is a page in the one document (host mode). In frame mode each page is a frame of its own, another realm: one Shiki per page then means one per frame, which a page can't share.
