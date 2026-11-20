+++
id = "01a1f89e-246e-7337-9254-b1fc9221ff5c"
number = 25
title = "The host gives the code highlighter; the host’s page highlights for every frame"
status = "approved"
tasks = ["CMD-262"]
author = "claude:79612601"
reviewers = []
created = 2026-11-20T11:01:16Z
updated = 2026-11-20T11:16:04Z
+++

The host's code highlighter, as [the design](docs/host-highlighter.md) decides (CR-21, decision 7 included): the host gives the sandbox its highlighter, in both page modes, and the host's page highlights for every frame, Shiki by default.

What to check:

- **The option:** `highlight`, a `CodeHighlighter`, on `HostRuntimeOptions` (so `createSandbox` takes it). Without it, Shiki from the frames' `highlight.js`, as before, now loaded into the host's page in frame mode too.
- **Protocol version 2** (`messages.ts`, `protocol.schema.json`, `docs/sandbox-protocol.md`): `highlight {id, code, language}` up, `highlighted {id, lines?}` down, and the level's `timeout`, how long a frame waits before a block keeps its text.
- **Only what was sent** (decision 4): `SentNodes` (new, `sent-nodes.ts`) holds the keys of the trees, splices and parts a parent sent a frame; `PageImages` now uses it too, so images and code blocks share one check. `forSent` answers only a block whose text and language were sent; a permissive frame passes its strict children's requests up as its own, and the host's page checks them against the part it sent.
- **One block a task** (decision 6): `oneBlockATask` chains the window's requests, a task apart, across every frame.
- **The frames:** `frame/runtime.ts` loads no highlighter; their policies drop `'wasm-unsafe-eval'` (decision 7). The spec's §15 note says a policy *may* allow WebAssembly, which still holds: no spec change.
- **A code renderer's block is answered too,** since its fallback shows it as a code block; the design's line is updated to say so.
- **The build:** `highlight.js`'s exports had been kept only because the runtime imported Shiki; `preserveEntrySignatures: "strict"` keeps them now.
- **`@compmd/render`:** `codeBlocks` (new export) lists a tree's blocks as a highlighter is asked for them; `nextTask` moved to its own module and is exported, for the sandbox's queue.
- **Release notes** go in with the release (the protocol's version 2: a host's page and the frames it serves upgrade together), as 0.8.0's did.

Tests: the protocol's ends (ids, answers in any order, a forged request, a failing highlighter, no answer within the timeout), the schema, `SentNodes`, the queue and the check; in Chrome and WebKit, the host's highlighter in both modes, light and dark, asked only for the blocks sent, a block in a framed use's slot, the default Shiki in both modes as before, and a strict frame that can't compile WebAssembly.

Checks: `cargo make ci`, `cargo make site`.
