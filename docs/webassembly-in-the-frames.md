# WebAssembly in the frames

**Status: accepted (2026-11-19); decision 1 reversed by [the host's code highlighter](host-highlighter.md), decision 7 (CMD-262, 2026-11-20).** The frames no longer highlight code themselves: they ask the host's page for each block's tokens, and their policies no longer allow `'wasm-unsafe-eval'`. Decision 2's engine rule still holds where Shiki runs, in the host's page. Whether the sandbox's frame documents (`strict.html`, `permissive.html`) allow WebAssembly, so the page's frame can highlight code with Shiki's Oniguruma engine, and what else was weighed. It builds on [running components in the sandbox](running-components-in-the-sandbox.md), whose threats (T1 to T10) it goes through, and on [the spike CMD-252](../spikes/highlight-perf/README.md), which measured the engines.

## Decisions

1. **Both frame documents allow compiling WebAssembly:** `'wasm-unsafe-eval'` in their `script-src`, and nothing else in their policies changes. Against the sandbox's threat model it gives component code nothing it hasn't already: a strict frame runs any code it writes itself today (`blob:` modules), and WebAssembly reaches nothing but through that code (*What it allows*, below). What it adds is the browser's WebAssembly engine as more surface for a browser bug, which every web page already has.
2. **The highlighter uses Oniguruma, its WebAssembly inlined in `highlight.js`** (`@shikijs/engine-oniguruma/wasm-inlined`), since a strict frame can fetch nothing, a `.wasm` file included, **except in Chromium, where it uses the JavaScript engine** (revised 2026-11-19, after measuring each frame's cost, *Open points*). One rule for the frames and for a host-mode page:
   - **Chromium** (Chrome, Edge, WebView2) runs the JavaScript engine fast, and there Oniguruma only costs more: 87 ms a frame against 32 ms, and its 230 KB. It is told by `navigator.userAgentData`, which only Chromium's browsers have, not by parsing the user agent string.
   - **Elsewhere** (WebKit, and browsers not measured, such as Firefox) Oniguruma, which is fast in every browser measured: in WebKit it takes the first TypeScript block from about a second to 60 ms, and its tokens were right where the JavaScript engine's weren't (CMD-261).
   - **A wrong guess costs time, never highlighting:** a Chromium where `userAgentData` is hidden (it is a secure context's) gets Oniguruma; a browser that refuses compiling gets the JavaScript engine (a host's own frame documents with another policy, a host page without `'wasm-unsafe-eval'`).
   - **The same document may be highlighted a little differently** in Chromium and elsewhere: the JavaScript engine leaves out a grammar's patterns it can't translate.
3. **No worker, no change to the protocol, no change to the spec's rules.** §15's informative note on browsers gains one line: the frames allow WebAssembly, which reaches no more than their scripts.

The alternatives, and why each comes second, are in *What else was weighed*.

## Terms

- **`'wasm-unsafe-eval'`**: a source of a content security policy's `script-src` that lets the page compile WebAssembly from bytes (`WebAssembly.compile`, `instantiate`, `Module`), and nothing else: it doesn't allow `eval` or `new Function`, which `'unsafe-eval'` would.
- **Oniguruma**: the regex library TextMate grammars are written for, built to WebAssembly by Shiki (`@shikijs/engine-oniguruma`, 622 KB, 232 KB gzipped).
- **The JavaScript engine**: Shiki's engine that turns a grammar's Oniguruma regexes into JavaScript ones (`@shikijs/engine-javascript`), which the frames use today, since they can't compile WebAssembly.

## Why

In frame mode the page's frame highlights its document's code blocks itself. In WebKit the JavaScript engine compiles each regex the first time it runs: the first TypeScript block blocks the page about a second, and new rules first matched later block again (in the spike, 1.1 s on opening and 0.9 s over a skim, with one Shiki). Oniguruma's first block takes 30 ms and nothing after it is long. In WebKit the frames run in the host page's process, so a frame's long frame is the host's too. Chromium is fast with either.

Host mode gets Oniguruma from the host page's own policy (CMD-257). This design is about the frames, where documents' own code runs.

## What it allows

What compiling WebAssembly in a frame lets component code do that its JavaScript can't, threat by threat:

| # | Threat | With WebAssembly |
|---|---|---|
| T1 | Strict code sends what it sees | No change. WebAssembly has no I/O of its own: it calls only the JavaScript functions it is given, so every request still goes through the frame's policy (`default-src 'none'`). |
| T2 | Strict code reads the host's session | No change: the frame's origin is still opaque. |
| T3 | Strict code navigates or opens a window | No change: the `sandbox` attribute is the same. |
| T4 | One document's code reads another's | No change: memory is the frame's own. Sharing memory between agents needs `SharedArrayBuffer`, which a frame gets only when its page is cross-origin isolated, which the frames aren't. |
| T5 | Strict code makes a frame with a network | No change: `frame-src 'none'`. |
| T6 | Permissive code widens its network | No change. |
| T7 | Code gets `host` it wasn't given | No change: `host` is given by the parent, in messages. |
| T8 | A module changed after the host checked it | No change: the bytes compiled are data in a script whose file the host checked, or that the frame built itself, as it can build JavaScript today. |
| T9 | A component runs forever | No change: a loop in WebAssembly is stopped as one in JavaScript is, by removing the frame (Chrome), and as little in WebKit, where a spinning frame already freezes the host's page ([what a browser can't stop](running-components-in-the-sandbox.md#threat-model)). |
| T10 | A fake prompt over the host's interface | No change. |

**Code from data.** The frames refuse `eval`, but allow `blob:` modules, which is how they run the source they compile: so strict code can already turn any text into code it runs. Compiling WebAssembly is one more way of doing that, no wider.

**Spec §15's rules for a strict sandbox** (only the contract; no network; no session; no navigation; can be stopped) are kept, by the table above.

**Timing.** Side channels such as Spectre need a fine timer. WebAssembly's fine timers come from threads sharing memory, which needs `SharedArrayBuffer` and so cross-origin isolation, which the frames don't have; `performance.now()` is the same coarse timer for both languages. What WebAssembly adds is faster code to time with, which JavaScript's optimizing compilers already give.

**Memory.** A WebAssembly memory can grow to 4 GB, as a JavaScript program can allocate; running a page out of memory is the same threat as running it forever (T9).

**The accepted risk** is the WebAssembly engine itself: a browser bug in its compiler is surface that JavaScript alone doesn't reach. Every web page allows WebAssembly by default; the frames would only stop refusing it.

## What else was weighed

| | A. WebAssembly in the frames (proposed) | B. The host's page tokenizes for its frames | C. A worker in the frame | D. The JavaScript engine, as today |
|---|---|---|---|---|
| What changes | `'wasm-unsafe-eval'` in the two policies | a message asking the host for tokens, and its answer; the frame draws them (CMD-251 made a highlighter give tokens) | `worker-src` in the policies, a worker document of the frames' own | nothing |
| WebKit, first TypeScript block | 30 ms, in the frame | as in host mode: 30 ms with Oniguruma in the host's page, in the host's thread | the wait of the engine in the worker (1.1 s with JavaScript's, 0.1 s with Oniguruma's, which needs A too) but no long frame | about 1 s, in the frame, which in WebKit is the host's process |
| Its risk | the WebAssembly engine as surface (above) | the host's page tokenizes untrusted text for code that may ask for anything: it must answer only blocks of trees it sent the frame, as it does for images; a grammar bug runs in the host's realm, which host mode accepts already | a worker's own policy, which a worker made from a URL takes from its response and not from the frame: one more document to keep closed | none new |
| Other costs | Oniguruma loaded and compiled in each frame (not measured per frame) | a round trip per block; each frame's highlighting waits on the host's thread | a worker per frame | the long frames stay |

**B** is the one worth keeping in mind: it would give frames one Shiki, the host's, for every frame of the page, where A has one per frame. It moves untrusted text into the host's realm and makes the host answer frames' requests, so it is a protocol change with checks of its own; A is one line of policy. If A's cost per frame turns out large (many small frames in a page), B is the next step.

**C** needs A as well to be fast (the spike's worker with the JavaScript engine still waits 1.1 s), so it adds a worker's policy to A's for smoothness A already gives.

**D** keeps the frames as they are and the second of jank in WebKit.

## What it means

- **The policies** (`npm-packages/sandbox/src/build/policy.ts`): `'wasm-unsafe-eval'` in `common`'s `script-src`, for both documents; their tests say so.
- **The highlighter** (`npm-packages/sandbox/src/highlight/highlight.ts`), in the frames and in a host-mode page: Oniguruma inlined, the JavaScript engine in Chromium and when compiling is refused (decision 2).
- **The browser tests** (Q8 of the sandbox's design) gain one: a strict component that compiles and runs WebAssembly given `fetch` still reaches no origin (T1).
- **The serving contract** is unchanged: its header policy is `sandbox allow-scripts` only, and the documents' own policies are in their `<meta>`.
- **The protocol** is unchanged.
- **The spec:** §15's informative note on browsers, one line (decision 3). No rule changes.

## Open points

1. **What Oniguruma costs a frame** (measured 2026-11-19, CMD-260; [the spike](../spikes/highlight-perf/README.md), *Each frame's first block*): twelve frames, one after the other, each a document of its own that makes the sandbox's highlighter and highlights one TypeScript block, the files from the cache after the first.

   | | First frame | Each frame after | Longest after | Blocked, all twelve | Long frames |
   |---|---|---|---|---|---|
   | WebKit, JavaScript engine | 660 ms | 31 ms | 436 ms | 1230 ms | 4–5 |
   | WebKit, Oniguruma | 56 ms | 84 ms | 122 ms | 312 ms | 9–10 |
   | Chromium, JavaScript engine | 49 ms | 32 ms | 65 ms | 0 | 0 |
   | Chromium, Oniguruma | 63 ms | 87 ms | 126 ms | 50 ms | 2–4 |

   - **Oniguruma's cost is each frame's own:** about 85 ms a frame in both browsers, compiling its WebAssembly again, where the JavaScript engine's frames after the first take about 30 ms (WebKit keeps the regexes it compiled for the frames after; its first frame pays 0.66 s).
   - **In WebKit, for twelve frames, Oniguruma blocks a quarter as long** (0.3 s against 1.2 s) and no frame takes over 130 ms, against 0.4–0.6 s; but every frame is long (over 50 ms), where most of the JavaScript engine's aren't.
   - **So it doesn't scale:** at about 35 ms blocked per frame, Oniguruma blocks longer than the JavaScript engine from about 35 frames on, in WebKit. In Chromium it is slower at any number.
   - This is what alternative B avoids: one Oniguruma, the host's, for every frame.
   - **Decided:** the JavaScript engine in Chromium, Oniguruma elsewhere (decision 2). A page of many frames in WebKit (host mode with dozens of strict uses that highlight code) still pays about 85 ms a frame; B is kept for it, as a task of its own (CMD-262).

## Follow-ups

- CMD-260: the policies, the highlighter's choice of engine (decision 2) in the frames and in a host-mode page, the browser test above, and §15's informative note with its changelog entry.
- CMD-262: alternative B, for pages of many frames.
