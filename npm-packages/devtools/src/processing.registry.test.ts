// Packs from the registry, in the worker: a document renders with what it has, its missing packs pending (spec §6.4),
// asked for at once on opening and, for an import typed, once typing pauses; when they arrive the document opens
// again with them, their built modules given as code, and when they can't be had, with them F026.
import { initSync, OpenDocumentJson, type PackRelease, type Request } from "@compmd/processor";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { beforeAll, describe, expect, it, vi } from "vitest";
import { Processing, type Fetched, type Resolution } from "./processing.ts";

beforeAll(() => {
  initSync({ module: readFileSync(createRequire(import.meta.url).resolve("@compmd/processor/compmd_wasm_bg.wasm")) });
});

const open = (request: Request) => new OpenDocumentJson(request);

/** A document importing `@kit/hello@1` and using its component. */
const DOCUMENT = '---\nmd:\n  import:\n    - from: "@kit/hello@1"\n      names: [hello]\n---\n\n:: hello[Hello, as text] {value="Hello"}\n';
const request = (text = DOCUMENT): Request => ({ path: "a.comp.md", files: { "a.comp.md": text }, now: "2026-11-02T09:00:00Z", origins: true });

/** `@kit/hello` 1.0.0 as the registry gives it: one component, a built module given by its place. */
const HELLO: PackRelease = {
  version: "1.0.0",
  integrity: "sha512-hello",
  files: {
    "package.json": JSON.stringify({ name: "@kit/hello", version: "1.0.0", compmd: "./index.comp.md" }),
    "index.comp.md": '---\nmd:\n  import:\n    - from: ./dist/hello.js\n      names: [hello]\n  export:\n    pack: "@kit/hello"\n    version: "1.0.0"\n    names: [hello]\n---\n\n```compmd component=hello signature\nparams:\n  value: string\n```\n',
    "dist/hello.js": "",
  },
};
const CODE = new Blob(["export default (target, { params }) => { target.textContent = params.value; };"]);

/** A registry that has `@kit/hello`, and records each request it was asked to resolve. */
function registry() {
  const asked: Request[] = [];
  const resolve = (given: Request): Promise<Resolution> => {
    asked.push(given);
    return Promise.resolve({
      request: { ...given, packs: { ...given.packs, "@kit/hello": [HELLO] } },
      modules: (place) => (place.pack === "@kit/hello" ? { code: CODE } : undefined),
      problems: [],
    });
  };
  return { asked, resolve };
}

const codes = (answer: ReturnType<Processing["handle"]>) => (answer.type === "processed" ? answer.diagnostics.map(({ code }) => code) : []);

describe("Processing, with a registry", () => {
  it("renders the missing pack's uses pending, without F026, fetches it at once, and opens with it, its module as code", async () => {
    const { asked, resolve } = registry();
    const fetched: Fetched[] = [];
    const processing = new Processing(open, { resolve, onFetched: (each) => fetched.push(each), pause: 5 });
    const first = processing.handle({ type: "open", request: request(), caret: 0 });
    expect(codes(first)).toEqual([]);
    expect(first.type === "processed" && first.splices[0]).toContain('"type":"componentPending"');
    await vi.waitFor(() => {
      expect(fetched).toEqual([{ again: true, problems: [] }]);
    });
    expect(asked).toHaveLength(1);
    // The editor opens the document again, with the same request: the worker adds the packs it fetched.
    const again = processing.handle({ type: "open", request: request(), caret: 0 });
    expect(codes(again)).toEqual([]);
    expect(again).toMatchObject({ components: [{ name: "hello", module: { pack: "@kit/hello", version: "1.0.0", path: "dist/hello.js" } }] });
    expect(again.type === "processed" && again.modules).toEqual({ "@kit/hello@1.0.0/dist/hello.js": CODE });
    // Nothing is missing any more, so nothing is asked for again.
    await new Promise((resolved) => setTimeout(resolved, 20));
    expect(asked).toHaveLength(1);
  });

  it("asks for an import typed only as it is once typing pauses, not for each name on the way", async () => {
    const { asked, resolve } = registry();
    const processing = new Processing(open, { resolve, pause: 30 });
    const text = DOCUMENT.replace("@kit/hello@1", "@kit/h@1");
    // An opened document's imports are asked for at once: it is a document, not one being typed.
    processing.handle({ type: "open", request: request(text), caret: 0 });
    expect(asked).toHaveLength(1);
    const at = text.indexOf("@kit/h") + "@kit/h".length;
    ["e", "l", "l", "o"].forEach((letter, i) => processing.handle({ type: "edits", edits: [{ start: at + i, end: at + i, text: letter }], caret: 0 }));
    await vi.waitFor(() => {
      expect(asked).toHaveLength(2);
    });
    expect(asked[1]?.files?.["a.comp.md"]).toContain('from: "@kit/hello@1"');
  });

  it("says what it couldn't fetch, and asks for it only once", async () => {
    const asked: Request[] = [];
    const fetched: Fetched[] = [];
    const resolve = (given: Request): Promise<Resolution> => {
      asked.push(given);
      return Promise.resolve({ request: given, modules: () => undefined, problems: [{ pack: "@kit/hello", message: "the registry has no version 1" }] });
    };
    const processing = new Processing(open, { resolve, onFetched: (each) => fetched.push(each), pause: 5 });
    processing.handle({ type: "open", request: request(), caret: 0 });
    await vi.waitFor(() => {
      expect(fetched).toEqual([{ again: true, problems: ["@kit/hello: the registry has no version 1"] }]);
    });
    // Opened again, as the editor does: no longer pending, so F026.
    expect(codes(processing.handle({ type: "open", request: request(), caret: 0 }))).toEqual(["F026"]);
    processing.handle({ type: "edits", edits: [{ start: DOCUMENT.length, end: DOCUMENT.length, text: "\nMore.\n" }], caret: 0 });
    await new Promise((resolved) => setTimeout(resolved, 20));
    expect(asked).toHaveLength(1);
  });

  it("fetches nothing without a registry", async () => {
    const processing = new Processing(open);
    expect(codes(processing.handle({ type: "open", request: request(), caret: 0 }))).toEqual(["F026"]);
    await new Promise((resolved) => setTimeout(resolved, 20));
    expect(codes(processing.handle({ type: "open", request: request(), caret: 0 }))).toEqual(["F026"]);
  });
});
