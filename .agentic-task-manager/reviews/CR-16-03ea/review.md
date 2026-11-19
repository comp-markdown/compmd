+++
id = "01a1f332-b998-770d-aaed-d866e39003ea"
number = 16
title = "Oniguruma in the frames, the JavaScript engine in Chromium"
status = "open"
tasks = ["~CMD-260"]
author = "claude:79612601"
reviewers = []
created = 2026-11-19T09:45:50Z
updated = 2026-11-19T09:45:50Z
+++

The frames allow compiling WebAssembly, and one rule picks the highlighter's engine wherever a page renders: the JavaScript engine in Chromium, Oniguruma elsewhere where the page compiles it ([WebAssembly in the frames](docs/webassembly-in-the-frames.md), decision 2 as revised).

## What to check

- **The policies** (`npm-packages/sandbox/src/build/policy.ts`): `'wasm-unsafe-eval'` in `common`'s `script-src`, both documents; nothing else changes (`policy.test.ts`).
- **The rule** (`highlight.ts`, `engineFor(webAssembly, page)`): Oniguruma when the highlighter may use it, the browser isn't Chromium's (`"userAgentData" in navigator`, which only Chromium's browsers have, in a secure context) and the page compiles an empty module; otherwise the JavaScript engine. A Chromium that hides it gets Oniguruma: time, never highlighting. The frames' runtime now asks for it (`shikiHighlight({ webAssembly: true })`), as `framesHighlight` does since CMD-257, which this changes in Chromium.
- **Security:** `test/strict.test.ts`'s new T1 case: a strict frame instantiates a module whose start function calls an imported `send`, which fetches; the fetch is refused and the server sees no request.
- **The spec:** §15's informative note on browsers, one sentence; its changelog entry under *Unreleased*, not breaking, no rule changes. This is the line the accepted design names.
- **Docs:** the sandbox's design (Q3's table), its README (which engine runs where, and that Chromium may highlight a little differently), `docs/hosting.md` and the Tauri guide (outside Chromium).
- **Tests:** the rule's unit test for Chromium, another browser and a page that refuses; in the browser tests, the engine Chrome and WebKit run. `cargo make ci` and `cargo make site` pass.

## Measured

Frame mode's first TypeScript block in WebKit: 660 ms with the JavaScript engine, 56 ms with Oniguruma (the spike's *Each frame's first block*, frames without `strict.html`'s policy, the same work). Pages of many frames in WebKit still pay about 85 ms a frame: CMD-262.
