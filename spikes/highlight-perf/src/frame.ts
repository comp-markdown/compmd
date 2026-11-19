// What a frame of the spike runs. `?frames=N` (CMD-260): the sandbox's highlighter, made in the frame, as each of a
// page's frames makes its own, and the time to its first block's tokens, which carries loading Shiki and its engine
// and compiling it: Oniguruma's WebAssembly with `webAssembly`, the JavaScript engine's regexes without.
// `?framepage=N` (CMD-267): a frame of a page of frames, highlighting its blocks with a highlighter of its own, or by
// asking the page that made it over a port.
import type { CodeHighlighter, CodeLines } from "@compmd/render";
import { shikiHighlight } from "@compmd/sandbox";

export async function firstTokens(code: string, webAssembly: boolean): Promise<number> {
  const started = performance.now();
  await shikiHighlight({ webAssembly })(code, "ts");
  return performance.now() - started;
}

/** A request for a block's tokens, and its answer, as the page and its frames send them over a port. */
export interface TokensRequest {
  id: number;
  code: string;
  language: string;
}
export interface TokensAnswer {
  id: number;
  lines: CodeLines | undefined;
}

/** A highlighter that asks the page over `port` for each block's tokens. */
function askingThePage(port: MessagePort): CodeHighlighter {
  const waiting = new Map<number, (lines: CodeLines | undefined) => void>();
  port.onmessage = ({ data }: MessageEvent<TokensAnswer>) => {
    waiting.get(data.id)?.(data.lines);
    waiting.delete(data.id);
  };
  let next = 0;
  return (code, language) =>
    new Promise((resolve) => {
      const id = next++;
      waiting.set(id, resolve);
      port.postMessage({ id, code, language } satisfies TokensRequest);
    });
}

/** How long a frame took: to its first block's tokens, and to all of them. */
export interface FrameReport {
  firstMs: number;
  allMs: number;
}

/**
 * Highlights `blocks` of TypeScript one after another, as the renderer's queue asks for them: with the page's
 * highlighter over `port` when there is one, with one of the frame's own (the sandbox's, choosing its engine) when not.
 */
export async function highlightBlocks(blocks: readonly string[], port: MessagePort | undefined): Promise<FrameReport> {
  const started = performance.now();
  const highlight = port ? askingThePage(port) : shikiHighlight({ webAssembly: true });
  let firstMs: number | undefined;
  for (const code of blocks) {
    await highlight(code, "ts");
    firstMs ??= performance.now() - started;
  }
  return { firstMs: firstMs ?? 0, allMs: performance.now() - started };
}
