+++
id = "01a1f2c3-198f-762d-86ae-9bc7e2bf7683"
number = 9
title = "Highlighters give tokens, and the renderer draws them"
status = "open"
tasks = ["~CMD-251"]
author = "claude:79612601"
reviewers = []
created = 2026-11-19T07:43:54Z
updated = 2026-11-19T07:43:54Z
+++

A highlighter gives tokens, and the renderer draws them into its own lines, so how a code block is drawn (its line elements, `data-line`) stays the renderer's own.

## What to check

- **The contract** (`npm-packages/render/src/highlight.ts`): `CodeHighlighter = (code, language) => CodeLines | undefined | Promise<…>`; a token is `{ text, color?: { light, dark }, italic?, bold?, underline? }`. `highlightCode` draws tokens given at once or later, drops those that come after a newer text's, and keeps the text when they aren't its text, when the highlighter gives none, throws or rejects. `CODE_LINE` is no longer exported.
- **The sandbox** (`npm-packages/sandbox/src/highlight/`): `shikiHighlight` gives Shiki's tokens through `codeTokens` (`tokens.ts`); its drawing is gone. `framesHighlight` passes the frames' highlighter's tokens on.
- **The blog** (`npm-packages/blog/src/highlight.ts`): a built page keeps Shiki's own HTML (`style` per token, its dark colours as custom properties, which its theme's CSS uses), so it now highlights the rendered page as a step after `render`, not through the page's `highlight`. Its output is as before.
- **The spike's page** keeps its own drawing, so its variants run as they were measured.
- **Docs:** the renderer's and the sandbox's READMEs, and the Developer's Guide.

## Breaking

A host's own highlighter returns tokens instead of writing into `target`. The packages' version notes are written at release; they should name this.

## Checks

`cargo make ci` and `cargo make site` pass. New `highlight.test.ts` in the renderer; the sandbox's highlighter tests now check tokens; the browser tests of code blocks pass in both modes.
