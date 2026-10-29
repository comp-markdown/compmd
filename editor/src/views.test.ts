import { initSync, type Request } from "@compmd/processor";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { beforeAll, describe, expect, it } from "vitest";
import { viewOf, ViewsClient, type View, type ViewRequest, type ViewsPort } from "./views.ts";

beforeAll(() => {
  initSync({ module: readFileSync(createRequire(import.meta.url).resolve("@compmd/processor/compmd_wasm_bg.wasm")) });
});

const request = (text: string): Request => ({ path: "a.comp.md", files: { "a.comp.md": text }, now: "2026-10-29T09:00:00Z" });

describe("viewOf", () => {
  it("gives both trees as indented JSON", () => {
    const view = viewOf({ request: request("# Hi\n\n:: chip[x]\n"), view: "trees" });
    if (view.view !== "trees" || !("parse" in view)) throw new Error(JSON.stringify(view));
    expect(JSON.parse(view.parse)).toMatchObject({ type: "root", children: [{ type: "heading" }, { type: "componentLeaf", name: "chip" }] });
    expect(JSON.parse(view.expanded)).toMatchObject({ type: "root" });
    expect(view.parse).toContain('\n  "children"');
  });

  it("gives what a plain GFM renderer shows", () => {
    expect(viewOf({ request: request(":: chip[x]\n"), view: "plain" })).toEqual({ view: "plain", html: "<p>:: chip[x]</p>\n" });
  });

  it("says why a request can't be processed", () => {
    expect(viewOf({ request: { path: "a", now: "never" }, view: "trees" })).toMatchObject({ view: "trees", failed: expect.any(String) as string });
  });
});

class FakeWorker implements ViewsPort {
  readonly received: ViewRequest[] = [];
  #listener: ((event: { data: View }) => void) | undefined;
  postMessage(message: ViewRequest): void {
    this.received.push(message);
  }
  addEventListener(_type: "message", listener: (event: { data: View }) => void): void {
    this.#listener = listener;
  }
  answer(data: View): void {
    this.#listener?.({ data });
  }
}

describe("ViewsClient", () => {
  it("asks once at a time, and only for the latest document and view", () => {
    const worker = new FakeWorker();
    const answers: View[] = [];
    const client = new ViewsClient(worker, (view) => answers.push(view));
    client.request(request("a"), "trees");
    client.request(request("b"), "trees");
    client.request(request("c"), "plain");
    expect(worker.received.map(({ request }) => request.files?.["a.comp.md"])).toEqual(["a"]);
    worker.answer({ view: "trees", parse: "1", expanded: "1" });
    expect(worker.received.map(({ request, view }) => [request.files?.["a.comp.md"], view])).toEqual([
      ["a", "trees"],
      ["c", "plain"],
    ]);
    worker.answer({ view: "plain", html: "" });
    expect(answers).toHaveLength(2);
    expect(worker.received).toHaveLength(2);
  });
});
