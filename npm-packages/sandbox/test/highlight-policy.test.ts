// The frames' highlighter in a host's page (host mode), under the page's own policy: Oniguruma where it allows
// WebAssembly (`'wasm-unsafe-eval'`, as `@compmd/vite`'s `cspFor` gives it) and the browser isn't Chromium's, the
// JavaScript engine otherwise, and the right tokens either way. Each page is a frame of this one with that policy, which loads `highlight.js` from where
// the frames are served, as `framesHighlight` does.

import type { CodeLines } from "@compmd/render";
import { afterEach, describe, expect, it } from "vitest";
import { engineFor } from "../src/highlight/highlight.ts";
import { frames } from "./harness.ts";

const placed: HTMLIFrameElement[] = [];
afterEach(() => {
  for (const frame of placed.splice(0)) frame.remove();
});

const CODE = "export async function load(url: string): Promise<number> {\n  return (await fetch(url)).status; // done\n}";

/** What a page with the policy `scriptSrc` gets: whether it compiles WebAssembly, and the frames' highlighter's tokens. */
function highlightedUnder(scriptSrc: string): Promise<{ compiles: boolean; lines: CodeLines | undefined }> {
  const url = new URL("highlight.js", frames).href;
  const frame = document.body.appendChild(document.createElement("iframe"));
  placed.push(frame);
  const id = String(Math.random());
  frame.srcdoc = `<meta http-equiv="Content-Security-Policy" content="script-src 'unsafe-inline' ${new URL(frames).origin} ${scriptSrc}">
<script type="module">
  const id = ${JSON.stringify(id)};
  try {
    const { shikiHighlight } = await import(${JSON.stringify(url)});
    const lines = await shikiHighlight({ webAssembly: true })(${JSON.stringify(CODE)}, "ts");
    const compiles = await WebAssembly.compile(new Uint8Array([0, 97, 115, 109, 1, 0, 0, 0])).then(() => true, () => false);
    parent.postMessage({ id, compiles, lines }, "*");
  } catch (error) {
    parent.postMessage({ id, error: String(error) }, "*");
  }
</script>`;
  return new Promise((resolve, reject) => {
    const heard = ({ data }: MessageEvent<{ id: string; compiles: boolean; lines: CodeLines | undefined; error?: string }>) => {
      if (data.id !== id) return;
      removeEventListener("message", heard);
      if (data.error !== undefined) reject(new Error(data.error));
      else resolve({ compiles: data.compiles, lines: data.lines });
    };
    addEventListener("message", heard);
  });
}

/** The block's text, as `lines` give it. */
const textOf = (lines: CodeLines | undefined) => (lines ?? []).map((line) => line.map(({ text }) => text).join("")).join("\n");
/** The colour of the token `text`, in each theme. */
const colorOf = (lines: CodeLines | undefined, text: string) => lines?.flat().find((token) => token.text === text)?.color;

describe("the engine a browser runs", () => {
  it("is the JavaScript engine in Chrome and Oniguruma in WebKit, on a page that compiles WebAssembly", async () => {
    const chrome = navigator.userAgent.includes("Chrome");
    expect(await engineFor(true)).toBe(chrome ? "javascript" : "oniguruma");
  });
});

describe("code highlighted in a host's page", () => {
  it("under a policy that allows WebAssembly and under one that doesn't", { timeout: 30_000 }, async () => {
    const [allowed, refused] = await Promise.all([highlightedUnder("'wasm-unsafe-eval'"), highlightedUnder("")]);
    expect([allowed.compiles, refused.compiles]).toEqual([true, false]);
    expect([textOf(allowed.lines), textOf(refused.lines)]).toEqual([CODE, CODE]);
    // GitHub's colours: a keyword, a type and a comment, in each theme.
    expect(colorOf(allowed.lines, "async")).toEqual({ light: "#D73A49", dark: "#F97583" });
    expect(colorOf(allowed.lines, "Promise")).toEqual({ light: "#6F42C1", dark: "#B392F0" });
    expect(colorOf(allowed.lines, "// done")).toEqual({ light: "#6A737D", dark: "#6A737D" });
    expect(colorOf(refused.lines, "async")).toEqual({ light: "#D73A49", dark: "#F97583" });
  });
});
