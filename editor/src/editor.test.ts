import type { Change } from "@compmd/render";
import type { FrameConnection, FrameMessage, SpliceOptions } from "@compmd/render/frame";
import { describe, expect, it } from "vitest";
import type { Example } from "./documents.ts";
import { Editor, type Frame } from "./editor.ts";
import type { ComponentFailed, ComponentsMessage, FromWorker, ToWorker } from "./protocol.ts";
import type { WorkerPort } from "./session.ts";

class FakeWorker implements WorkerPort {
  readonly received: ToWorker[] = [];
  #listener: ((event: { data: FromWorker }) => void) | undefined;
  postMessage(message: ToWorker): void {
    this.received.push(message);
  }
  addEventListener(_type: "message", listener: (event: { data: FromWorker }) => void): void {
    this.#listener = listener;
  }
  answer(data: FromWorker): void {
    this.#listener?.({ data });
  }
}

type Listener = (message: FrameMessage) => void;

/** A frame that records what it is sent, and lets the test send what a frame would. */
class FakeFrame implements Frame {
  readonly sent: (["splice", readonly unknown[], Change | undefined, SpliceOptions | undefined] | ["components", ComponentsMessage])[] = [];
  readonly #listeners = new Map<string, Listener[]>();
  #failed: ((failed: ComponentFailed) => void) | undefined;
  readonly connection: FrameConnection = {
    render: () => {
      throw new Error("the editor sends splices");
    },
    splice: (splices, change, options) => this.sent.push(["splice", splices, change, options]),
    theme: () => undefined,
    on: (type, listener) => {
      this.#listeners.set(type, [...(this.#listeners.get(type) ?? []), listener as Listener]);
    },
    close: () => undefined,
  };
  send(message: ComponentsMessage): void {
    this.sent.push(["components", message]);
  }
  onFailed(listener: (failed: ComponentFailed) => void): void {
    this.#failed = listener;
  }
  emit(message: FrameMessage): void {
    for (const listener of this.#listeners.get(message.type) ?? []) listener(message);
  }
  fail(failed: ComponentFailed): void {
    this.#failed?.(failed);
  }
}

const example: Example = { name: "A", request: { path: "a.comp.md", files: { "a.comp.md": "Hello", "b.csv": "x" }, now: "2026-10-22T09:00:00Z" } };
const opened = { ...example.request, origins: true };
const processed = (splices: string[], extra: Partial<Extract<FromWorker, { type: "processed" }>> = {}): FromWorker => ({ type: "processed", splices, opened: false, diagnostics: [], ms: 1, ...extra });
const diagnostic = (file?: string) => ({
  code: "F002",
  severity: "error" as const,
  message: "no such component",
  range: { start: { line: 3, column: 4 } },
  stage: "resolve" as const,
  ...(file ? { file } : {}),
});

function setup() {
  const worker = new FakeWorker();
  const frame = new FakeFrame();
  let changes = 0;
  const editor = new Editor(worker, frame, () => changes++);
  return { worker, frame, editor, changes: () => changes };
}

describe("Editor", () => {
  it("opens the document with origins, then sends each input as an edit, and passes each answer's splices on", () => {
    const { worker, frame, editor } = setup();
    expect(editor.open(example)).toBe("Hello");
    expect(worker.received).toEqual([{ type: "open", request: opened }]);
    worker.answer(processed(["whole"], { opened: true, components: [{ name: "c", language: "js", code: "x" }], ms: 2 }));
    expect(frame.sent).toEqual([
      ["components", { type: "components", components: [{ name: "c", language: "js", code: "x" }] }],
      ["splice", ["whole"], { text: "Hello", diagnostics: [] }, { replace: true }],
    ]);
    editor.beforeInput({ start: 5, end: 5 });
    editor.input("Hello!");
    expect(worker.received[1]).toEqual({ type: "edits", edits: [{ start: 5, end: 5, text: "!" }] });
    worker.answer(processed(["typed"], { diagnostics: [diagnostic(), diagnostic("b.comp.md")] }));
    expect(frame.sent[2]).toEqual(["splice", ["typed"], { text: "Hello!", diagnostics: [{ code: "F002", line: 3, column: 4 }] }, { replace: false }]);
    expect(editor.state.diagnostics).toHaveLength(2);
    expect(editor.state.processing.values).toEqual([2, 1]);
  });

  it("gives the document as it is now, and counts the answers that changed it", () => {
    const { worker, editor } = setup();
    expect(editor.document).toBeUndefined();
    editor.open(example);
    editor.beforeInput({ start: 5, end: 5 });
    editor.input("Hello!");
    expect(editor.document).toEqual({ ...opened, files: { "a.comp.md": "Hello!", "b.csv": "x" }, inputs: {} });
    worker.answer(processed(["whole"], { opened: true }));
    worker.answer(processed(["typed"]));
    expect(editor.state.updates).toBe(2);
  });

  it("makes a diagnostic's fix as an edit, only in the text its diagnostic was found in", () => {
    const { worker, editor } = setup();
    editor.open(example);
    worker.answer(processed(["whole"], { opened: true }));
    const fix = { range: { start: { line: 1, column: 1 }, end: { line: 1, column: 6 } }, text: "Howdy" };
    expect(editor.fix(fix)).toBe("Howdy");
    expect(worker.received[1]).toEqual({ type: "edits", edits: [{ start: 0, end: 5, text: "Howdy" }] });
    expect(editor.fix(fix), "its diagnostics are of the text before it").toBe("Howdy");
    expect(worker.received).toHaveLength(2);
  });

  it("times each render from its first key press to the frame painted", () => {
    const { worker, frame, editor } = setup();
    editor.open(example, 0);
    worker.answer(processed(["whole"], { opened: true }));
    frame.emit({ type: "rendered", ms: 3 });
    expect(editor.state.painted.count).toBe(1);
    expect(editor.state.rendering.values).toEqual([3]);
    frame.emit({ type: "rendered", ms: 3 });
    expect(editor.state.painted.count).toBe(1);
  });

  it("opens the document again, with the text as it is, when the page sets an input", () => {
    const { worker, frame, editor } = setup();
    editor.open(example);
    worker.answer(processed(["whole"], { opened: true }));
    editor.beforeInput({ start: 0, end: 0 });
    editor.input("Oh Hello");
    worker.answer(processed(["typed"]));
    frame.emit({ type: "set", input: "region", value: "eu" });
    expect(worker.received[2]).toEqual({
      type: "open",
      request: { ...opened, files: { "a.comp.md": "Oh Hello", "b.csv": "x" }, inputs: { region: "eu" } },
    });
  });

  it("lists what the worker and the frame couldn't do, once each, until the components change", () => {
    const { worker, frame, editor, changes } = setup();
    editor.open(example);
    worker.answer({ type: "failed", message: "bad request" });
    expect(editor.state.problems).toEqual(["bad request"]);
    frame.fail({ type: "component-failed", component: "chart", message: "does not compile" });
    frame.fail({ type: "component-failed", component: "chart", message: "does not compile" });
    frame.emit({ type: "error", reason: "threw", component: "board", message: "boom" });
    frame.emit({ type: "error", reason: "no-container", message: "no container" });
    expect(editor.state.problems).toEqual(["bad request", "chart: does not compile", "board: boom"]);
    editor.beforeInput({ start: 0, end: 0 });
    editor.input("x");
    worker.answer(processed(["typed"], { components: [] }));
    expect(editor.state.problems).toEqual([]);
    expect(changes()).toBeGreaterThan(4);
  });
});
