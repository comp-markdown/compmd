// Tokenizing off the page's thread: the page posts a block's code, the worker answers with its tokens and how long
// tokenizing took. One worker for the page; its Shiki is made on the first message, by the engine it names.

import type { HighlighterCore } from "@shikijs/core";
import { loadShiki, tokensOf, type Engine } from "./shiki.ts";

let shiki: Promise<{ shiki: HighlighterCore; ms: number }> | undefined;

addEventListener("message", (event: MessageEvent<{ id: number; code: string; engine: Engine }>) => {
  const { id, code, engine } = event.data;
  shiki ??= loadShiki(engine);
  void shiki.then(({ shiki: loaded, ms: setup }) => {
    const started = performance.now();
    const lines = tokensOf(loaded, code);
    postMessage({ id, lines, ms: performance.now() - started, setup });
  });
});
