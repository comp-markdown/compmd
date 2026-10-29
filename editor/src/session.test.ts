import { describe, expect, it } from "vitest";
import type { FromWorker, ToWorker } from "./protocol.ts";
import { Session, type Update, type WorkerPort } from "./session.ts";

/** A worker the test answers by hand. */
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

const processed = (marker: string, opened = false): Extract<FromWorker, { type: "processed" }> => ({ type: "processed", splices: [marker], opened, diagnostics: [], ms: 1 });
const request = { path: "a.comp.md", files: { "a.comp.md": "ab" }, now: "2026-10-22T09:00:00Z" };

function setup() {
  const worker = new FakeWorker();
  const updates: Update[] = [];
  const failures: string[] = [];
  const session = new Session(worker, (update) => updates.push(update), (message) => failures.push(message));
  return { worker, updates, failures, session };
}

describe("Session", () => {
  it("sends one edit at a time, and merges those made while the worker is busy", () => {
    const { worker, updates, session } = setup();
    session.open(request, "ab", 1);
    session.edit({ start: 2, end: 2, text: "c" }, "abc", 2);
    expect(worker.received).toHaveLength(1);
    worker.answer(processed("opened", true));
    expect(updates[0]).toEqual({ splices: ["opened"], opened: true, diagnostics: [], text: "ab", since: 1, ms: 1 });
    expect(worker.received[1]).toEqual({ type: "edits", edits: [{ start: 2, end: 2, text: "c" }] });
    session.edit({ start: 3, end: 3, text: "d" }, "abcd", 3);
    session.edit({ start: 4, end: 4, text: "e" }, "abcde", 4);
    worker.answer(processed("one"));
    expect(updates[1]).toMatchObject({ splices: ["one"], opened: false, text: "abc", since: 2 });
    expect(worker.received[2]).toEqual({ type: "edits", edits: [{ start: 3, end: 3, text: "d" }, { start: 4, end: 4, text: "e" }] });
    worker.answer({ ...processed("merged"), splices: ["d", "e"], components: [] });
    expect(updates[2]).toMatchObject({ splices: ["d", "e"], text: "abcde", since: 3, components: [] });
    expect(worker.received).toHaveLength(3);
  });

  it("drops answers for a document opened over, and edits not sent yet", () => {
    const { worker, updates, session } = setup();
    session.open(request, "ab");
    worker.answer(processed("first"));
    session.edit({ start: 0, end: 0, text: "x" }, "xab");
    session.edit({ start: 0, end: 0, text: "y" }, "yxab");
    session.open(request, "other");
    worker.answer(processed("stale"));
    worker.answer(processed("second"));
    expect(updates.map((update) => update.text)).toEqual(["ab", "other"]);
    expect(worker.received.map((message) => message.type)).toEqual(["open", "edits", "open"]);
  });

  it("reports what the worker couldn't do", () => {
    const { worker, failures, session } = setup();
    session.open(request, "ab");
    worker.answer({ type: "failed", message: "bad request" });
    expect(failures).toEqual(["bad request"]);
  });
});
