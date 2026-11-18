+++
id = "01a1f03c-ac29-708f-be97-bb0ccb911a3b"
number = 6
title = "A code block's lines, an element each, numbered"
status = "open"
tasks = ["~CMD-250"]
author = "claude:79612601"
reviewers = []
created = 2026-11-18T19:57:50Z
updated = 2026-11-18T19:57:50Z
+++

A code block's lines, each an element of its own, numbered with their lines in the text, so a host can size a line: atm's diff keeps its two sides level when a line wraps (ATM-588). Option 1 of the three discussed: document lines, renumbered when an edit moves a block.

## What to check

- **`npm-packages/render/src/code-lines.ts`** (new): `drawCodeLines` draws `<span class="compmd-line">` per line, its line break inside it; `codeOf` gives a placed `<pre>`'s `<code>` and its first line (moved out of `lines.ts`); `numberCodeLines` sets `data-line`, writing only what changed.
- **`blocks.ts`**: after a render, numbers the blocks drawn again or moved since they were numbered (`numbered`), and only those that have a code block (`summary.code`). `placed()`'s per-block part is `#placedOf`.
- **`markdown-views.ts`**: `CodeBlockView` draws the lines in place of the text. **`styles.ts`**: `.compmd-line { display: block; }` in the base rules.
- **The highlighter** (`npm-packages/sandbox/src/highlight/highlight.ts`): each line's tokens go into its line's element, which keeps its attributes; a `<code>` whose lines aren't an element each keeps its text. `CodeHighlighter`'s doc and both READMEs give the new contract; `CODE_LINE` is exported.
- **`lines.ts` `byVisualLine`**: boxes are one visual line when they overlap by more than half the shorter's height, not by any amount. WebKit's text boxes are 15.13px high every 15px, so a wrapped line, a code block's or a paragraph's, came out as one box.
- **Checks that looked for any span** to see a block highlighted now look for the tokens in its lines: the sandbox's hosting check (`check.ts`), the Tauri example's, and the code blocks' browser tests.

## Not done

`lines()` is unchanged otherwise: it already gives a code block's lines; atm saw none because it processes a diff's run as task text, without origins.

## Breaking

A host's own highlighter that replaces the `<code>`'s children drops the line elements. The renderer's next release notes say so.

## Checks

`cargo make ci`, `cargo make site` and `cargo make tauri-check` pass; the new browser test passes in Chrome and WebKit.
