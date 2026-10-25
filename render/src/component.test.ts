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
