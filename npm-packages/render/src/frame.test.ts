import { describe, expect, it, vi } from "vitest";
import type { Mount } from "./contract.ts";
import { connectFrame, PROTOCOL_VERSION, serveFrame, type FrameMessage, type Port } from "./frame.ts";
import type { ExpandedRoot } from "./tree.ts";

/** Two ends of an in-memory channel, delivering each message on the next task as a `MessagePort` does. */
function channel(): [Port, Port] {
  const listeners: [((event: { data: unknown }) => void)[], ((event: { data: unknown }) => void)[]] = [[], []];
  const end = (mine: 0 | 1): Port => ({
    postMessage: (data) => {
      setTimeout(() => {
        for (const listener of listeners[mine === 0 ? 1 : 0]) listener({ data });
      }, 0);
    },
    addEventListener: (_type, listener) => {
      listeners[mine].push(listener);
    },
    removeEventListener: (_type, listener) => {
      listeners[mine] = listeners[mine].filter((l) => l !== listener);
    },
  });
  return [end(0), end(1)];
}

const settle = () => new Promise((resolve) => setTimeout(resolve, 20));

const tree = (value: string): ExpandedRoot => ({ type: "root", children: [{ type: "paragraph", children: [{ type: "text", value }] }] });

describe("the host and frame protocol", () => {
  it("sends renders once the frame is ready, and reports sets, errors and paints", async () => {
    const [host, frame] = channel();
    const container = document.body.appendChild(document.createElement("main"));
    const connection = connectFrame(host);
    const received: FrameMessage[] = [];
    for (const type of ["ready", "set", "error", "rendered"] as const) connection.on(type, (message) => received.push(message));
    connection.render(tree("queued before ready"));

    const chart: Mount = (_target, { set }) => {
      queueMicrotask(() => {
        set("selected", "eu");
      });
    };
    const served = serveFrame(frame, { container, components: { chart } });
    await vi.waitFor(() => {
      expect(container.textContent).toBe("queued before ready");
    });

    connection.render({
      type: "root",
      children: [
        { type: "componentLeaf", name: "chart", params: [{ key: "selected", breakpoint: null, value: "all", input: "region" }], fallback: [] },
        { type: "componentLeaf", name: "gone", params: [], fallback: [] },
      ],
    });
    // `rendered` follows the next animation frame, which a busy machine may take longer than `settle` to give.
    await vi.waitFor(() => {
      expect(received.filter((message) => message.type === "rendered")).toHaveLength(2);
    });
    expect(received[0]).toEqual({ type: "ready", version: PROTOCOL_VERSION });
    expect(received).toContainEqual({ type: "set", input: "region", value: "eu" });
    expect(received).toContainEqual(expect.objectContaining({ type: "error", component: "gone", reason: "missing" }));

    served.close();
    container.remove();
  });

  it("sends nothing to a frame with another protocol version", async () => {
    const [host, frame] = channel();
    const connection = connectFrame(host);
    const versions: number[] = [];
    connection.on("ready", ({ version }) => versions.push(version));
    connection.render(tree("x"));
    const seen: unknown[] = [];
    frame.addEventListener("message", ({ data }) => seen.push(data));
    frame.postMessage({ type: "ready", version: 99 });
    await settle();
    expect(versions).toEqual([99]);
    expect(seen).toEqual([]);
  });

  it("applies splices, as objects or JSON text, and one that replaces every block", async () => {
    const [host, frame] = channel();
    const container = document.body.appendChild(document.createElement("main"));
    const served = serveFrame(frame, { container });
    const connection = connectFrame(host);
    const rendered: number[] = [];
    connection.on("rendered", ({ ms }) => rendered.push(ms));
    connection.render(tree("one"));
    const second = { from: 1, to: 1, nodes: tree("two").children, lines: 0, offset: 0 };
    connection.splice([second, JSON.stringify({ ...second, from: 2, to: 2, nodes: tree("three").children })]);
    await settle();
    expect(container.textContent).toBe("onetwothree");
    connection.splice([JSON.stringify({ from: 0, to: 0, nodes: tree("again").children, lines: 0, offset: 0 }), { ...second, nodes: tree("more").children }], { text: "again\n\nmore\n" }, { replace: true });
    await settle();
    expect(container.textContent).toBe("againmore");
    // Each render reports once its frame has painted, which may come after 20 ms while other tests run.
    await vi.waitFor(() => {
      expect(rendered).toHaveLength(3);
    });
    served.close();
    container.remove();
  });

  it("ignores messages of the host's own on the same port", async () => {
    const [host, frame] = channel();
    const container = document.body.appendChild(document.createElement("main"));
    const served = serveFrame(frame, { container });
    const connection = connectFrame(host);
    const rendered: number[] = [];
    connection.on("rendered", ({ ms }) => rendered.push(ms));
    connection.render(tree("kept"));
    await settle();
    host.postMessage({ type: "host-style", style: "github" });
    host.postMessage({ type: "components", components: [] });
    await settle();
    expect(container.textContent).toBe("kept");
    expect(rendered, "a paint for the render alone").toHaveLength(1);
    served.close();
    container.remove();
  });

  it("reveals and measures what a source offset drew, and reports the source of a click once asked", async () => {
    const [host, frame] = channel();
    const container = document.body.appendChild(document.createElement("main"));
    const served = serveFrame(frame, { container });
    const connection = connectFrame(host);
    const pointed: unknown[] = [];
    connection.on("pointed", ({ source }) => pointed.push(source));
    const origin = { start: { line: 1, column: 1, offset: 0 }, end: { line: 1, column: 5, offset: 4 }, occurrence: 0 };
    connection.render({ type: "root", children: [{ type: "paragraph", origin, children: [{ type: "text", value: "Text", origin }] }] });
    connection.reveal(2);
    const rects = await connection.rectsAt(2);
    expect(rects.map((rect) => Object.keys(rect))).toEqual([["x", "y", "width", "height"]]);
    expect(container.querySelector("p")?.classList.contains("compmd-revealed")).toBe(true);
    expect(await connection.rectsAt(40)).toEqual([]);

    container.querySelector("p")?.click();
    connection.point(true);
    await connection.rectsAt(0);
    container.querySelector("p")?.click();
    await vi.waitFor(() => {
      expect(pointed).toEqual([{ start: origin.start, end: origin.end }]);
    });
    served.close();
    container.remove();
  });

  it("passes on only a well-formed source and boxes, whatever the frame sent", async () => {
    const [host, frame] = channel();
    const connection = connectFrame(host);
    const pointed: unknown[] = [];
    connection.on("pointed", ({ source }) => pointed.push(source));
    frame.addEventListener("message", ({ data }) => {
      const { type, id } = data as { type: string; id?: number };
      if (type === "rects") frame.postMessage({ type: "rects", id, rects: [{ x: 1, y: 2, width: 3, height: 4 }, { x: "1" }, null] });
    });
    frame.postMessage({ type: "ready", version: PROTOCOL_VERSION });
    frame.postMessage({ type: "pointed", source: { start: { line: "1", column: 1 }, end: {} } });
    frame.postMessage({ type: "pointed", source: { start: { line: 1, column: 1, offset: 0, extra: "x" }, end: { line: 1, column: 2 } } });
    expect(await connection.rectsAt(0)).toEqual([{ x: 1, y: 2, width: 3, height: 4 }]);
    expect(pointed).toEqual([{ start: { line: 1, column: 1, offset: 0 }, end: { line: 1, column: 2 } }]);
    const unanswered = connection.rectsAt(0);
    connection.close();
    expect(await unanswered).toEqual([]);
  });

  it("gives the frame's lines a batch at a time, as the loop asks, and says when they may have moved", async () => {
    const boxes = vi.spyOn(Range.prototype, "getClientRects").mockReturnValue([{ x: 0, y: 0, width: 10, height: 18 }] as unknown as DOMRectList);
    const [host, frame] = channel();
    const container = document.body.appendChild(document.createElement("main"));
    const served = serveFrame(frame, { container });
    const connection = connectFrame(host);
    const asked: number[] = [];
    frame.addEventListener("message", ({ data }) => {
      const { type, from } = data as { type: string; from?: number };
      if (type === "lines" && from !== undefined) asked.push(from);
    });
    let layouts = 0;
    connection.on("layout", () => (layouts += 1));
    // A paragraph on each of the lines 1 to 250.
    const paragraph = (line: number) => {
      const origin = { start: { line, column: 1, offset: line * 10 }, end: { line, column: 5, offset: line * 10 + 4 }, occurrence: 0 };
      return { type: "paragraph", origin, children: [{ type: "text", value: "Line", origin }] };
    };
    connection.render({ type: "root", children: Array.from({ length: 250 }, (_, i) => paragraph(i + 1)) });
    const lines: number[] = [];
    for await (const { line } of connection.lines()) lines.push(line);
    expect(lines).toEqual(Array.from({ length: 250 }, (_, i) => i + 1));
    expect(asked).toEqual([1, 201]);
    await vi.waitFor(() => {
      expect(layouts).toBeGreaterThan(0);
    });

    asked.length = 0;
    for await (const { line } of connection.lines(10)) if (line === 12) break;
    expect(asked, "a loop that stops asks for no more").toEqual([10]);
    served.close();
    container.remove();
    boxes.mockRestore();
  });

  it("passes on only lines that are whole numbers from 1, after those given, whatever the frame sent", async () => {
    const [host, frame] = channel();
    const connection = connectFrame(host);
    const rect = { x: 1, y: 2, width: 3, height: 4 };
    frame.addEventListener("message", ({ data }) => {
      const { type, id, from } = data as { type: string; id?: number; from?: number };
      if (type !== "lines") return;
      // The same line again and again, which would keep a loop that believed it going.
      if (from === 1) frame.postMessage({ type: "lines", id, lines: [{ line: 1, rects: [rect, { x: "1" }] }, { line: 0, rects: [] }, { line: 2.5, rects: [] }, null], done: false });
      else frame.postMessage({ type: "lines", id, lines: [{ line: 1, rects: [rect] }], done: false });
    });
    frame.postMessage({ type: "ready", version: PROTOCOL_VERSION });
    const lines = [];
    for await (const line of connection.lines()) lines.push(line);
    expect(lines).toEqual([{ line: 1, rects: [rect] }]);

    const unanswered = connection.lines(5).next();
    connection.close();
    expect(await unanswered).toEqual({ done: true, value: undefined });
  });

  it("applies theme tokens sent by the host", async () => {
    const [host, frame] = channel();
    const container = document.body.appendChild(document.createElement("main"));
    const served = serveFrame(frame, { container });
    connectFrame(host).theme({ "--compmd-accent": "teal" });
    await settle();
    expect(container.style.getPropertyValue("--compmd-accent")).toBe("teal");
    served.close();
    container.remove();
  });
});
