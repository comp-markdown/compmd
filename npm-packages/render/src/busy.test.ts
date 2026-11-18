import { afterEach, describe, expect, it, vi } from "vitest";
import { BUSY_MS } from "./busy.ts";
import type { Mount } from "./contract.ts";
import { connectFrame, serveFrame, type Port } from "./frame.ts";
import { createPage } from "./page.ts";
import type { ExpandedRoot } from "./tree.ts";

/** A component that draws once `drawing` resolves, as a diagram drawn asynchronously does, and says so with `busy`. */
function late(drawing: Promise<unknown>): Mount {
  return (target, { busy }) => {
    busy(
      drawing.then(() => {
        target.append("drawn");
      }),
    );
  };
}

const use = (name: string): ExpandedRoot => ({ type: "root", children: [{ type: "componentLeaf", name, params: [], fallback: [] }] });

/** A promise with its resolve, for a drawing the test ends when it likes. */
function drawing(): { promise: Promise<void>; end: () => void } {
  let end: () => void = () => undefined;
  const promise = new Promise<void>((resolve) => {
    end = resolve;
  });
  return { promise, end };
}

let root: HTMLElement;
afterEach(() => {
  vi.useRealTimers();
  root.remove();
});

describe("drawn", () => {
  it("resolves at once when no component is drawing", async () => {
    root = document.body.appendChild(document.createElement("main"));
    const page = createPage(root, { components: { chart: () => undefined } });
    page.render(use("chart"));
    await expect(page.drawn()).resolves.toBeUndefined();
  });

  it("resolves once every drawing a component gave `busy` has settled, those of a later render too", async () => {
    root = document.body.appendChild(document.createElement("main"));
    const [first, second] = [drawing(), drawing()];
    const page = createPage(root, { components: { a: late(first.promise), b: late(second.promise) } });
    page.render(use("a"));
    let done = false;
    void page.drawn().then(() => (done = true));
    page.render({ type: "root", children: [...use("a").children, ...use("b").children] });
    first.end();
    await Promise.resolve();
    expect(done).toBe(false);
    second.end();
    await vi.waitFor(() => {
      expect(done).toBe(true);
    });
    expect(root.textContent).toBe("drawndrawn");
  });

  it("counts a drawing that fails as settled", async () => {
    root = document.body.appendChild(document.createElement("main"));
    const page = createPage(root, { components: { chart: late(Promise.reject(new Error("no"))) } });
    page.render(use("chart"));
    await expect(page.drawn()).resolves.toBeUndefined();
  });

  it("stops waiting for a drawing that never settles after BUSY_MS", async () => {
    vi.useFakeTimers();
    root = document.body.appendChild(document.createElement("main"));
    const page = createPage(root, { components: { chart: late(new Promise(() => undefined)) } });
    page.render(use("chart"));
    let done = false;
    void page.drawn().then(() => (done = true));
    await vi.advanceTimersByTimeAsync(BUSY_MS - 1);
    expect(done).toBe(false);
    await vi.advanceTimersByTimeAsync(1);
    expect(done).toBe(true);
  });
});

describe("drawn, through a frame", () => {
  it("says for how many renders and splices the frame's page has drawn", async () => {
    const listeners: [((event: { data: unknown }) => void)[], ((event: { data: unknown }) => void)[]] = [[], []];
    const end = (mine: 0 | 1): Port => ({
      postMessage: (data) => setTimeout(() => {
        for (const listener of listeners[mine === 0 ? 1 : 0]) listener({ data });
      }, 0),
      addEventListener: (_type, listener) => listeners[mine].push(listener),
      removeEventListener: () => undefined,
    });
    root = document.body.appendChild(document.createElement("main"));
    const later = drawing();
    const served = serveFrame(end(1), { container: root, components: { chart: late(later.promise) } });
    const connection = connectFrame(end(0));
    const seen: number[] = [];
    connection.on("drawn", ({ seen: count }) => seen.push(count));
    connection.render(use("chart"));
    connection.render(use("chart"));
    await new Promise((resolve) => setTimeout(resolve, 30));
    expect(seen, "not while the chart draws").toEqual([]);
    later.end();
    await vi.waitFor(() => {
      expect(seen).toEqual([2]);
    });
    served.close();
  });
});
