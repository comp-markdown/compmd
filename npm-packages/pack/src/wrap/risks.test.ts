// What a strict frame refuses, found in a built module: each kind once, with the text that found it; names that are no
// fetch, such as SVG's namespace, aren't.
import { describe, expect, it } from "vitest";
import { risksOf } from "./risks.ts";

describe("risksOf", () => {
  it("warns of another origin's URL, a worker, code made from text and WebGL, each once", () => {
    const code = 'fetch("https://cdn.example.com/draco.wasm");fetch("https://cdn.example.com/b");new Worker(u);eval(s);c.getContext("webgl2")';
    expect(risksOf("dist/viewer.js", code)).toEqual([
      "dist/viewer.js has a URL of another origin, which a strict frame can't fetch: `https://cdn.example.com/draco.wasm`",
      "dist/viewer.js has a worker, which a strict frame can't start: `new Worker(`",
      "dist/viewer.js has code made from text, which a strict frame refuses: `eval(`",
      'dist/viewer.js has WebGL, which a strict frame has no context for: `getContext("webgl2"`',
    ]);
  });

  it("takes a namespace for the name it is, and finds nothing in plain drawing code", () => {
    expect(risksOf("dist/qr.js", 'document.createElementNS("http://www.w3.org/2000/svg","svg");new Function;c.getContext("2d")')).toEqual([]);
  });
});
