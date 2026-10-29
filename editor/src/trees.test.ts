import { initSync, type Request } from "@compmd/processor";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { beforeAll, describe, expect, it } from "vitest";
import { treesOf, TreesClient, type Trees, type TreesPort, type TreesRequest } from "./trees.ts";

beforeAll(() => {
  initSync({ module: readFileSync(createRequire(import.meta.url).resolve("@compmd/processor/compmd_wasm_bg.wasm")) });
});

const request = (text: string): Request => ({ path: "a.comp.md", files: { "a.comp.md": text }, now: "2026-10-29T09:00:00Z" });

describe("treesOf", () => {
  it("gives both trees as indented JSON", () => {
    const trees = treesOf({ request: request("# Hi\n\n:: chip[x]\n") });
    if ("failed" in trees) throw new Error(trees.failed);
    expect(JSON.parse(trees.parse)).toMatchObject({ type: "root", children: [{ type: "heading" }, { type: "componentLeaf", name: "chip" }] });
    expect(JSON.parse(trees.expanded)).toMatchObject({ type: "root" });
    expect(trees.parse).toContain('\n  "children"');
  });

  it("says why a request can't be processed", () => {
    expect(treesOf({ request: { path: "a", now: "never" } })).toMatchObject({ failed: expect.any(String) as string });
  });
});

class FakeWorker implements TreesPort {
  readonly received: TreesRequest[] = [];
  #listener: ((event: { data: Trees }) => void) | undefined;
  postMessage(message: TreesRequest): void {
    this.received.push(message);
  }
  addEventListener(_type: "message", listener: (event: { data: Trees }) => void): void {
    this.#listener = listener;
  }
  answer(data: Trees): void {
    this.#listener?.({ data });
  }
}

describe("TreesClient", () => {
  it("asks once at a time, and only for the latest document", () => {
    const worker = new FakeWorker();
    const answers: Trees[] = [];
    const client = new TreesClient(worker, (trees) => answers.push(trees));
    client.request(request("a"));
    client.request(request("b"));
    client.request(request("c"));
    expect(worker.received.map(({ request }) => request.files?.["a.comp.md"])).toEqual(["a"]);
    worker.answer({ parse: "1", expanded: "1" });
    expect(worker.received.map(({ request }) => request.files?.["a.comp.md"])).toEqual(["a", "c"]);
    worker.answer({ parse: "2", expanded: "2" });
    expect(answers).toHaveLength(2);
    expect(worker.received).toHaveLength(2);
  });
});
