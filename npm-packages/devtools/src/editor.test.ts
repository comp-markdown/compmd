import type { HostComponent } from "@compmd/processor";
import type { Change } from "@compmd/render";
import type { ViewEvents, ViewSpliceOptions } from "@compmd/sandbox";
import { describe, expect, it } from "vitest";
import type { DevtoolsDocument } from "./document.ts";
import { Editor, type Frame } from "./editor.ts";
import type { FromWorker, ToWorker } from "./protocol.ts";
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

type Listener = (event: unknown) => void;

/** A page in its sandbox that records what it is sent, and lets the test report what the page would. */
class FakeFrame implements Frame {
  readonly sent: (["splice", readonly unknown[], Change | undefined, ViewSpliceOptions | undefined] | ["components", readonly HostComponent[]])[] = [];
  /** The offsets revealed, in code points. */
  readonly revealed: number[] = [];
  readonly #listeners = new Map<string, Listener[]>();
  reveal(offset: number): void {
    this.revealed.push(offset);
  }
  splice(splices: readonly unknown[], change?: Change, options?: ViewSpliceOptions): void {
    this.sent.push(["splice", splices, change, options]);
  }
  components(components: readonly HostComponent[]): Promise<void> {
    this.sent.push(["components", components]);
    return Promise.resolve();
  }
  on<T extends keyof ViewEvents>(type: T, listener: (event: ViewEvents[T]) => void): void {
    this.#listeners.set(type, [...(this.#listeners.get(type) ?? []), listener as Listener]);
  }
  emit<T extends keyof ViewEvents>(type: T, event: ViewEvents[T]): void {
    for (const listener of this.#listeners.get(type) ?? []) listener(event);
  }
}

const example: DevtoolsDocument = { request: { path: "a.comp.md", files: { "a.comp.md": "Hello", "b.csv": "x" }, now: "2026-10-22T09:00:00Z" } };
const opened = { ...example.request, origins: true };
const processed = (splices: string[], extra: Partial<Extract<FromWorker, { type: "processed" }>> = {}): FromWorker => ({ type: "processed", splices, opened: false, diagnostics: [], inputs: {}, ms: 1, ...extra });
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

describe("Editor's problems", () => {
  it("lists a use that failed until it draws again, and keeps another use's that still fails", () => {
    const { frame, editor } = setup();
    editor.open(example);
    frame.emit("error", { component: "mermaid", reason: "threw", code: "W118", message: "Parse error", use: "u1" });
    frame.emit("error", { component: "mermaid", reason: "threw", code: "W118", message: "Lexical error", use: "u2" });
    expect(editor.state.problems).toEqual(["W118 mermaid: Parse error", "W118 mermaid: Lexical error"]);
    frame.emit("recovered", { component: "mermaid", use: "u1" });
    expect(editor.state.problems).toEqual(["W118 mermaid: Lexical error"]);
    frame.emit("recovered", { component: "mermaid", use: "u9" });
    expect(editor.state.problems).toEqual(["W118 mermaid: Lexical error"]);
  });

  it("keeps a problem two uses share until both draw again", () => {
    const { frame, editor } = setup();
    editor.open(example);
    for (const use of ["u1", "u2"]) frame.emit("error", { component: "mermaid", reason: "threw", code: "W118", message: "Parse error", use });
    frame.emit("recovered", { component: "mermaid", use: "u1" });
    expect(editor.state.problems).toEqual(["W118 mermaid: Parse error"]);
    frame.emit("recovered", { component: "mermaid", use: "u2" });
    expect(editor.state.problems).toEqual([]);
  });
});

describe("Editor's caret on the page", () => {
  it("reveals where the reader moves the caret, in code points, but not where typing leaves it", () => {
    const { frame, editor } = setup();
    editor.open({ request: { ...example.request, files: { "a.comp.md": "😀 Hello" } } });
    editor.moveCaret(3);
    editor.moveCaret(3);
    editor.beforeInput({ start: 8, end: 8 });
    editor.input("😀 Hello!", 9);
    editor.moveCaret(9);
    editor.moveCaret(0);
    expect(frame.revealed).toEqual([2, 0]);
  });
});

describe("Editor", () => {
  it("opens the document with origins, then sends each input as an edit, and passes each answer's splices on", () => {
    const { worker, frame, editor } = setup();
    expect(editor.open(example)).toBe("Hello");
    expect(worker.received).toEqual([{ type: "open", request: opened, caret: 0 }]);
    const component: HostComponent = { name: "c", language: "js", code: "x", trust: { level: "strict" } };
    worker.answer(processed(["whole"], { opened: true, components: [component], ms: 2 }));
    expect(frame.sent).toEqual([
      ["components", [component]],
      ["splice", ["whole"], { text: "Hello", diagnostics: [] }, { replace: true, inputs: {} }],
    ]);
    editor.beforeInput({ start: 5, end: 5 });
    editor.input("Hello!", 6);
    expect(worker.received[1]).toEqual({ type: "edits", edits: [{ start: 5, end: 5, text: "!" }], caret: 6 });
    const inputs = { region: { type: "string", value: "eu" } };
    worker.answer(processed(["typed"], { diagnostics: [diagnostic(), diagnostic("b.comp.md")], inputs }));
    expect(frame.sent[2]).toEqual(["splice", ["typed"], { text: "Hello!", diagnostics: [{ code: "F002", line: 3, column: 4 }] }, { replace: false, inputs }]);
    expect(editor.state.diagnostics).toHaveLength(2);
    expect(editor.state.processing.values).toEqual([2, 1]);
  });

  it("loads the remote data the worker reports, asking about new origins in the page, and opens the document again with it", async () => {
    const worker = new FakeWorker();
    const asked: { urls: readonly string[]; configured: readonly string[] }[] = [];
    const load = (urls: readonly string[], { configured }: { configured: readonly string[] }) => {
      asked.push({ urls, configured });
      return Promise.resolve({ allowedOrigins: [...configured, "https://api.example.com"], remote: { "https://api.example.com/n.json": { text: "1", contentType: "application/json" } } });
    };
    const editor = new Editor(worker, new FakeFrame(), () => undefined, new Map(), load);
    const bundled = { "https://status.example.org/s.json": { text: "{}", contentType: "application/json" } };
    editor.open({ request: { ...example.request, allowedOrigins: ["https://status.example.org"], remote: bundled } });
    worker.answer(processed(["whole"], { opened: true }));
    worker.answer({ type: "remote", urls: ["https://status.example.org/s.json", "https://api.example.com/n.json"] });
    await Promise.resolve();
    await Promise.resolve();
    // What the document came with isn't fetched again; its configured origins are never asked about.
    expect(asked).toEqual([{ urls: ["https://api.example.com/n.json"], configured: ["https://status.example.org"] }]);
    const reopened = worker.received.at(-1);
    expect(reopened?.type).toBe("open");
    if (reopened?.type !== "open") return;
    expect(reopened.request.allowedOrigins).toEqual(["https://status.example.org", "https://api.example.com"]);
    expect(Object.keys(reopened.request.remote ?? {})).toEqual(["https://status.example.org/s.json", "https://api.example.com/n.json"]);
  });

  it("doesn't open the document again when the reader allowed nothing new", async () => {
    const worker = new FakeWorker();
    const load = () => Promise.resolve({ allowedOrigins: [], remote: {} });
    const editor = new Editor(worker, new FakeFrame(), () => undefined, new Map(), load);
    editor.open(example);
    worker.answer(processed(["whole"], { opened: true }));
    worker.answer({ type: "remote", urls: ["https://api.example.com/n.json"] });
    await Promise.resolve();
    await Promise.resolve();
    expect(worker.received).toHaveLength(1);
  });

  it("gives the document as it is now, and counts the answers that changed it", () => {
    const { worker, editor } = setup();
    expect(editor.document).toBeUndefined();
    editor.open(example);
    editor.beforeInput({ start: 5, end: 5 });
    editor.input("Hello!", 6);
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
    expect(editor.fix(fix)).toEqual({ text: "Howdy", caret: 5 });
    expect(worker.received[1]).toEqual({ type: "edits", edits: [{ start: 0, end: 5, text: "Howdy" }], caret: 5 });
    expect(editor.fix(fix), "its diagnostics are of the text before it").toBeUndefined();
    expect(worker.received).toHaveLength(2);
  });

  it("keeps the hint at the caret, and writes a value in its place only in the text it is of", () => {
    const { worker, editor, changes } = setup();
    editor.open(example);
    expect(editor.complete("Hi"), "no hint yet").toBeUndefined();
    const hint = { range: { start: { line: 1, column: 1 }, end: { line: 1, column: 6 } }, completions: [], diagnostics: [] };
    worker.answer(processed(["whole"], { opened: true, hint }));
    expect(editor.state.hint).toEqual(hint);
    editor.moveCaret(2);
    expect(worker.received[1]).toEqual({ type: "hint", caret: 2 });
    const before = changes();
    worker.answer({ type: "hint", hint: { ...hint, completions: [{ label: "Hi", text: "Hi" }] } });
    expect(editor.state.hint?.completions).toHaveLength(1);
    expect(changes()).toBe(before + 1);
    expect(editor.complete("Hi")).toEqual({ text: "Hi", caret: 2 });
    expect(worker.received[2]).toEqual({ type: "edits", edits: [{ start: 0, end: 5, text: "Hi" }], caret: 2 });
    expect(editor.complete("Yo"), "the hint is of the text before").toBeUndefined();
    worker.answer(processed(["typed"]));
    expect(editor.state.hint, "nothing at the caret now").toBeUndefined();
    editor.open(example);
    expect(editor.state.hint).toBeUndefined();
  });

  it("times each render from its first key press to the frame painted", () => {
    const { worker, frame, editor } = setup();
    editor.open(example, 0);
    worker.answer(processed(["whole"], { opened: true }));
    frame.emit("rendered", { ms: 3 });
    expect(editor.state.painted.count).toBe(1);
    expect(editor.state.rendering.values).toEqual([3]);
    frame.emit("rendered", { ms: 3 });
    expect(editor.state.painted.count).toBe(1);
  });

  it("opens the document again, with the text as it is, when the page sets an input", () => {
    const { worker, frame, editor } = setup();
    editor.open(example);
    worker.answer(processed(["whole"], { opened: true }));
    editor.beforeInput({ start: 0, end: 0 });
    editor.input("Oh Hello", 3);
    worker.answer(processed(["typed"]));
    frame.emit("set", { input: "region", value: "eu" });
    expect(worker.received[2]).toEqual({
      type: "open",
      request: { ...opened, files: { "a.comp.md": "Oh Hello", "b.csv": "x" }, inputs: { region: "eu" } },
      caret: 3,
    });
    // The inputs panel sets them the same way, keeping the others.
    worker.answer(processed(["whole"], { opened: true, inputs: { region: { type: "enum", members: ["all", "eu"], value: "eu" } } }));
    expect(editor.state.inputs).toEqual({ region: { type: "enum", members: ["all", "eu"], value: "eu" } });
    editor.setInput("live", true);
    expect(worker.received[3]).toMatchObject({ type: "open", request: { inputs: { region: "eu", live: true } } });
  });

  it("opens the document again when packs arrive from the registry, its fetched modules where the sandbox finds them first", () => {
    const worker = new FakeWorker();
    const frame = new FakeFrame();
    const modules = new Map<string, Blob>();
    const editor = new Editor(worker, frame, () => undefined, modules);
    editor.open(example);
    worker.answer(processed(["whole"], { opened: true }));
    // Not an answer: the session still waits for none.
    worker.answer({ type: "fetched", again: true, problems: ["@kit/other: the registry has no version 2"] });
    expect(worker.received[1]).toEqual({ type: "open", request: { ...opened, inputs: {} }, caret: 0 });
    const code = new Blob(["export default () => {};"]);
    const component: HostComponent = { name: "hello", language: "js", code: "", trust: { level: "strict" }, module: { pack: "@kit/hello", version: "1.0.0", path: "dist/hello.js" } };
    let found: Blob | undefined;
    frame.components = (components) => {
      found = modules.get("@kit/hello@1.0.0/dist/hello.js");
      frame.sent.push(["components", components]);
      return Promise.resolve();
    };
    worker.answer(processed(["whole"], { opened: true, components: [component], modules: { "@kit/hello@1.0.0/dist/hello.js": code } }));
    expect(found).toBe(code);
    // What couldn't be fetched stays listed when the components change.
    expect(editor.state.problems).toEqual(["@kit/other: the registry has no version 2"]);
    worker.answer({ type: "fetched", again: false, problems: [] });
    expect(worker.received).toHaveLength(2);
  });

  it("lists what the worker and the page couldn't do, once each, until the components change", () => {
    const { worker, frame, editor, changes } = setup();
    editor.open(example);
    worker.answer({ type: "failed", message: "bad request" });
    expect(editor.state.problems).toEqual(["bad request"]);
    frame.emit("error", { component: "chart", reason: "failed", message: "does not compile" });
    frame.emit("error", { component: "chart", reason: "failed", message: "does not compile" });
    frame.emit("error", { component: "board", reason: "threw", message: "boom" });
    // The page's own problem, such as breakpoints with no container, names no component.
    frame.emit("error", { component: "", reason: "threw", message: "no container" });
    expect(editor.state.problems).toEqual(["bad request", "chart: does not compile", "board: boom"]);
    editor.beforeInput({ start: 0, end: 0 });
    editor.input("x", 1);
    worker.answer(processed(["typed"], { components: [] }));
    expect(editor.state.problems).toEqual([]);
    expect(changes()).toBeGreaterThan(4);
  });
});
