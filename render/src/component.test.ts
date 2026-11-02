// The component layer on its own, as an editor's widget uses it: the caller renders the slots and the fallback.
import { describe, expect, it, vi } from "vitest";
import { mountComponent } from "./component.ts";
import type { Mount } from "./contract.ts";
import { Stylesheet } from "./styles.ts";
import type { ExpandedComponent } from "./tree.ts";

const node = (rows: number[], padding = "m"): ExpandedComponent => ({
  type: "componentLeaf",
  name: "chart",
  params: [
    { key: "rows", breakpoint: null, value: rows },
    { key: "padding", breakpoint: null, value: padding },
  ],
  fallback: [],
});

function setup(mount: Mount | undefined) {
  const widget = document.createElement("div");
  const stylesheet = new Stylesheet(document);
  const fallback = { text: "Sales chart", element: Object.assign(document.createElement("span"), { textContent: "Sales chart" }) };
  const onError = vi.fn();
  const handle = mountComponent(widget, node([1]), { mount, slots: { default: [] }, fallback, headingLevel: 2, onSet: () => undefined, onError, stylesheet });
  stylesheet.flush();
  return { widget, stylesheet, handle, onError };
}

describe("mountComponent", () => {
  it("mounts the component in a new box at the end of its target, its box styled by the common params", () => {
    const mount = vi.fn<Mount>(() => undefined);
    const { widget, stylesheet, handle } = setup(mount);
    expect(widget.lastElementChild).toBe(handle.element);
    expect(mount.mock.calls[0]?.[1]).toMatchObject({ params: { rows: [1] }, slots: { default: [] }, headingLevel: 2 });
    const name = [...handle.element.classList].find((c) => c.startsWith("compmd-b"));
    expect(stylesheet.text).toContain(`.${String(name)} { padding: var(--compmd-space-m, 1rem); }`);
    stylesheet.remove();
  });

  it("updates the component and its box, and destroys both", () => {
    const update = vi.fn();
    const destroy = vi.fn();
    const { widget, stylesheet, handle } = setup(() => ({ update, destroy }));
    handle.update(node([1, 2], "l"), { slots: {} });
    stylesheet.flush();
    expect(update).toHaveBeenCalledWith({ rows: [1, 2] }, {});
    expect(stylesheet.text).toContain("var(--compmd-space-l, 2rem)");
    handle.destroy();
    expect(destroy).toHaveBeenCalled();
    expect(widget.children).toHaveLength(0);
    stylesheet.remove();
  });

  it("shows the fallback when the component throws, and reports it", () => {
    const { handle, onError, stylesheet } = setup(() => {
      throw new Error("boom");
    });
    expect(handle.failed).toBe(true);
    expect(handle.element.textContent).toBe("Sales chart");
    expect(onError).toHaveBeenCalledWith(expect.objectContaining({ component: "chart", reason: "threw" }));
    stylesheet.remove();
  });

  it("shows the fallback when there is no component", () => {
    const { handle, onError, stylesheet } = setup(undefined);
    expect(handle.element.textContent).toBe("Sales chart");
    expect(onError).toHaveBeenCalledWith({ component: "chart", reason: "missing" });
    stylesheet.remove();
  });
});

describe("a component that fails", () => {
  /** A chart that draws its rows, and fails later on an empty list, as an asynchronous parse does. */
  const chart: Mount = (target, { params, fail }) => {
    const draw = (rows: unknown) => {
      if (Array.isArray(rows) && rows.length === 0) queueMicrotask(() => { fail("no rows to draw"); });
      target.textContent = `drawn ${JSON.stringify(rows)}`;
    };
    draw(params.rows);
    return { update: (next) => { draw(next.rows); } };
  };

  it("shows its fallback, with W118's message, until its next update draws again", async () => {
    const { handle, onError, stylesheet } = setup(chart);
    handle.update(node([]), { slots: {} });
    await Promise.resolve();
    expect(handle.element.textContent).toBe("Sales chart");
    expect(handle.failed).toBe(false);
    expect(onError).toHaveBeenCalledWith({ component: "chart", reason: "threw", error: new Error("no rows to draw") });
    handle.update(node([3]), { slots: {} });
    expect(handle.element.textContent).toBe("drawn [3]");
    stylesheet.remove();
  });

  it("is updated again after it threw while updating", () => {
    let throws = true;
    const update = vi.fn(() => {
      if (throws) throw new Error("bad rows");
    });
    const { handle, onError, stylesheet } = setup(() => ({ update }));
    handle.update(node([2]), { slots: {} });
    expect(handle.element.textContent).toBe("Sales chart");
    expect(onError).toHaveBeenCalledWith(expect.objectContaining({ reason: "threw", error: new Error("bad rows") }));
    throws = false;
    handle.update(node([3]), { slots: {} });
    expect(update).toHaveBeenCalledTimes(2);
    expect(handle.element.textContent).toBe("");
    stylesheet.remove();
  });

  it("is never updated after it threw while mounting", () => {
    const update = vi.fn();
    const { handle, stylesheet } = setup((_, { params }) => {
      if (Array.isArray(params.rows)) throw new Error("boom");
      return { update };
    });
    handle.update(node([2]), { slots: {} });
    handle.scheme("dark");
    expect(update).not.toHaveBeenCalled();
    expect(handle.failed).toBe(true);
    stylesheet.remove();
  });

  it("is ignored once the component is destroyed", async () => {
    const { handle, onError, stylesheet } = setup(chart);
    handle.update(node([]), { slots: {} });
    handle.destroy();
    await Promise.resolve();
    expect(onError).not.toHaveBeenCalled();
    stylesheet.remove();
  });
});

describe("the colour scheme", () => {
  it("is light unless the caller gives one, and a change updates the component with the same params", () => {
    const schemes: string[] = [];
    const { handle, stylesheet } = setup((_, context) => {
      schemes.push(context.scheme);
      return { update: (params) => { schemes.push(`${context.scheme} ${JSON.stringify(params.rows)}`); } };
    });
    handle.scheme("dark");
    handle.scheme("dark");
    expect(schemes).toEqual(["light", "dark [1]"]);
    stylesheet.remove();
  });

  it("draws a failed component again", () => {
    const draws: string[] = [];
    const { handle, stylesheet } = setup((target, { fail }) => {
      fail("not yet");
      return {
        update: () => {
          draws.push("update");
          target.textContent = "drawn";
        },
      };
    });
    expect(handle.element.textContent).toBe("Sales chart");
    handle.scheme("dark");
    expect(draws).toEqual(["update"]);
    expect(handle.element.textContent).toBe("drawn");
    stylesheet.remove();
  });
});
