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
  it("gives both trees", () => {
    const view = viewOf({ request: request("# Hi\n\n:: chip[x]\n"), view: "trees" });
    if (view.view !== "trees" || !("parse" in view)) throw new Error(JSON.stringify(view));
    expect(view.parse).toMatchObject({ type: "root", children: [{ type: "heading" }, { type: "componentLeaf", name: "chip" }] });
    expect(view.expanded).toMatchObject({ type: "root" });
  });

  it("gives what the frontmatter gives the render", () => {
    const view = viewOf({ request: request("---\ntitle: T\n---\n"), view: "data" });
    expect(view).toMatchObject({ view: "data", frontmatter: { doc: { title: "T" }, data: [], inputs: {} } });
  });

  it("gives the components the document can name", () => {
    const view = viewOf({ request: request(":: box[x]\n"), view: "components" });
    if (view.view !== "components" || !("components" in view)) throw new Error(JSON.stringify(view));
    const box = view.components.groups.flatMap((group) => group.components).find((component) => component.name === "box");
    expect(box?.uses).toEqual([{ start: { line: 1, column: 4 }, end: { line: 1, column: 7 } }]);
  });

  it("gives what a plain GFM renderer shows", () => {
    expect(viewOf({ request: request(":: chip[x]\n"), view: "plain" })).toEqual({ view: "plain", html: "<p>:: chip[x]</p>\n" });
  });

  it("gives the diagnostics, for the inspector", () => {
    const view = viewOf({ request: request(":: nope[x]\n"), view: "diagnostics" });
    expect(view).toMatchObject({ view: "diagnostics", diagnostics: [{ code: "F002" }] });
  });

  it("evaluates an expression in the document, or gives its F024", () => {
    const document = request("---\nteam: [Ana, Kenji]\n---\n");
    expect(viewOf({ request: document, view: "evaluate", id: 7, expression: "md.doc.team.size()" })).toEqual({ view: "evaluate", id: 7, result: { value: 2 } });
    expect(viewOf({ request: document, view: "evaluate", id: 8, expression: "md.doc.tem" })).toMatchObject({ view: "evaluate", id: 8, result: { error: { code: "F024" } } });
    expect(viewOf({ request: { path: "a", now: "never" }, view: "evaluate", id: 9, expression: "1" })).toMatchObject({ view: "evaluate", id: 9, failed: expect.any(String) as string });
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
    worker.answer({ view: "trees", parse: {}, expanded: {} });
    expect(worker.received.map(({ request, view }) => [request.files?.["a.comp.md"], view])).toEqual([
      ["a", "trees"],
      ["c", "plain"],
    ]);
    worker.answer({ view: "plain", html: "" });
    expect(answers).toHaveLength(2);
    expect(worker.received).toHaveLength(2);
  });

  it("keeps the diagnostics waiting apart from a tab's view, so neither drops the other", () => {
    const worker = new FakeWorker();
    const client = new ViewsClient(worker, () => undefined);
    client.request(request("a"), "trees");
    client.request(request("b"), "diagnostics");
    client.request(request("c"), "plain");
    worker.answer({ view: "trees", parse: {}, expanded: {} });
    worker.answer({ view: "diagnostics", diagnostics: [] });
    expect(worker.received.map(({ request, view }) => [request.files?.["a.comp.md"], view])).toEqual([
      ["a", "trees"],
      ["b", "diagnostics"],
      ["c", "plain"],
    ]);
  });

  it("sends every expression, in order, before the views that wait", () => {
    const worker = new FakeWorker();
    const client = new ViewsClient(worker, () => undefined);
    client.request(request("a"), "trees");
    client.request(request("b"), "plain");
    client.evaluate(request("b"), 1, "1 + 1");
    client.evaluate(request("b"), 2, "2 + 2");
    for (const answer of [{ view: "trees", parse: {}, expanded: {} }, ...[1, 2].map((id) => ({ view: "evaluate", id, result: { value: 0 } }))] as View[]) worker.answer(answer);
    expect(worker.received.map((asked) => (asked.view === "evaluate" ? asked.expression : asked.view))).toEqual(["trees", "1 + 1", "2 + 2", "plain"]);
  });
});
