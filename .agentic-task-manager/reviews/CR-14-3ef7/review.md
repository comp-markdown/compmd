+++
id = "01a1f2fb-b4ee-71b9-a099-c82d641a3ef7"
number = 14
title = "Oniguruma in host mode, where the page allows WebAssembly"
status = "open"
tasks = ["~CMD-257"]
author = "claude:79612601"
reviewers = []
created = 2026-11-19T08:45:44Z
updated = 2026-11-19T08:51:40Z
+++

A host-mode page highlights code with Oniguruma, as WebAssembly, where its policy lets it compile WebAssembly, and with the JavaScript engine where it doesn't. The frames are unchanged (CMD-260).

## What to check

- **The engine** (`npm-packages/sandbox/src/highlight/highlight.ts`): `shikiHighlight({ webAssembly: true })` asks the page with an 8-byte empty module (`compilesWebAssembly`) before loading Oniguruma's 230 KB, and falls back to the JavaScript engine; one engine for both of its Shikis. Without the option, as the frames' runtime calls it, nothing changes.
- **Host mode** (`from-frames.ts`): `framesHighlight` asks for it.
- **The policy:** `@compmd/vite`'s `cspFor` adds `'wasm-unsafe-eval'` to `script-src`; `docs/hosting.md` decision 10 says what it allows (compiling WebAssembly, nothing else) and that a host may leave it out; the Tauri example's policy, its README, the Tauri plugin's design and the Developer's Guide's copy of the policy have it.
- **The package:** `@shikijs/engine-oniguruma` 4.5.0, the version of Shiki's other packages; its WebAssembly is a chunk of its own in `dist/frames/`, loaded only where it is used.
- **Tests:** unit tests of the engine's choice and the check; `test/highlight-policy.test.ts` highlights in a page whose policy allows WebAssembly and in one whose doesn't, in Chrome and WebKit, with GitHub's colours for a keyword, a type and a comment; `cspFor`'s tests. `cargo make ci`, `cargo make tauri-check` and `cargo make site` pass.

## Measured

`spikes/highlight-perf/README.md`, *with Oniguruma*: in WebKit the opening's blocked time from about 1.2 s to 0.25 s and the skim's from 0.9 s to 0.15 s (medians; these runs varied more than before). Short of the spike's best (83 ms, 10 ms): the sandbox's highlighter makes two Shikis and loads grammars as blocks need them; CMD-259 is for what is left.

## A finding, apart

In WebKit the JavaScript engine's tokens for one TypeScript block were wrong in the browser test: its light theme's colours disagreed with its dark theme's, and with Oniguruma's and Chromium's. Frame mode still uses that engine. Seen once; not looked into.
