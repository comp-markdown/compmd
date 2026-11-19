// What a frame of `?frames=N` runs (CMD-260): the sandbox's highlighter, made in the frame, as each of a page's frames
// makes its own, and the time to its first block's tokens, which carries loading Shiki and its engine and compiling
// it: Oniguruma's WebAssembly with `webAssembly`, the JavaScript engine's regexes without.
import { shikiHighlight } from "@compmd/sandbox";

export async function firstTokens(code: string, webAssembly: boolean): Promise<number> {
  const started = performance.now();
  await shikiHighlight({ webAssembly })(code, "ts");
  return performance.now() - started;
}
