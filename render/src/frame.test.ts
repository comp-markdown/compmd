import { describe, expect, it } from "vitest";
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
    await settle();
    expect(container.textContent).toBe("queued before ready");

    connection.render({
      type: "root",
      children: [
        { type: "componentLeaf", name: "chart", params: [{ key: "selected", breakpoint: null, value: "all", input: "region" }], fallback: [] },
        { type: "componentLeaf", name: "gone", params: [], fallback: [] },
      ],
    });
    await settle();
    expect(received[0]).toEqual({ type: "ready", version: PROTOCOL_VERSION });
    expect(received.filter((message) => message.type === "rendered")).toHaveLength(2);
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
    expect(rendered).toHaveLength(3);
    served.close();
    container.remove();
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
