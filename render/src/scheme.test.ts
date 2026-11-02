// The colour scheme components are shown in (spec §7.4): read from what the page or region renders into, and given to
// its components when it changes.
import { afterEach, describe, expect, it, vi } from "vitest";
import type { Mount } from "./contract.ts";
import { rendersSlots } from "./nodes.ts";
import { createPage } from "./page.ts";
import { createRegion } from "./region.ts";
import { schemeOf } from "./scheme.ts";
import type { ExpandedComponent, ExpandedRoot } from "./tree.ts";

const chart: ExpandedComponent = { type: "componentLeaf", name: "chart", params: [], fallback: [] };
const tree: ExpandedRoot = { type: "root", children: [chart] };

/** A media query list for `prefers-color-scheme: dark` that a test switches. */
function preference(dark: boolean) {
  const listeners = new Set<() => void>();
  const list = {
    matches: dark,
    addEventListener: (_: string, listener: () => void) => listeners.add(listener),
    removeEventListener: (_: string, listener: () => void) => listeners.delete(listener),
  };
  vi.spyOn(window, "matchMedia").mockReturnValue(list as unknown as MediaQueryList);
  return {
    listeners,
    switchTo(next: boolean) {
      list.matches = next;
      for (const listener of listeners) listener();
    },
  };
}

/** A component that records the scheme it is shown in, at mount and at each update. */
function recording() {
  const schemes: string[] = [];
  const mount: Mount = (_, context) => {
    schemes.push(context.scheme);
    return { update: () => { schemes.push(context.scheme); } };
  };
  return { schemes, mount };
}

let root: HTMLElement;
function place(scheme: string) {
  root = document.body.appendChild(document.createElement("main"));
  root.style.colorScheme = scheme;
  return root;
}
afterEach(() => {
  root.remove();
  vi.restoreAllMocks();
});

describe("schemeOf", () => {
  it("is the element's colour scheme, the reader's preference where it allows both, and light without one", () => {
    preference(true);
    expect(schemeOf(place("dark"))).toBe("dark");
    expect(schemeOf(place("light"))).toBe("light");
    expect(schemeOf(place("light dark"))).toBe("dark");
    expect(schemeOf(place(""))).toBe("light");
  });
});

describe("a page's components", () => {
  it("are shown in the page's scheme, and follow the reader's preference", () => {
    const reader = preference(false);
    const { schemes, mount } = recording();
    const page = createPage(place("light dark"), { components: { chart: mount } });
    page.render(tree);
    reader.switchTo(true);
    expect(schemes).toEqual(["light", "dark"]);
    page.destroy();
    expect(reader.listeners.size).toBe(0);
  });

  it("follow a theme the host switches itself, when it says so or at the next render", () => {
    preference(false);
    const { schemes, mount } = recording();
    const page = createPage(place("light"), { components: { chart: mount } });
    page.render(tree);
    root.style.colorScheme = "dark";
    page.scheme();
    root.style.colorScheme = "light";
    page.render(tree);
    expect(schemes).toEqual(["light", "dark", "light", "light"]);
    page.destroy();
  });

  it("are drawn again when printing starts and ends", () => {
    preference(false);
    const { schemes, mount } = recording();
    const page = createPage(place("dark"), { components: { chart: mount } });
    page.render(tree);
    root.style.colorScheme = "light";
    window.dispatchEvent(new Event("beforeprint"));
    root.style.colorScheme = "dark";
    window.dispatchEvent(new Event("afterprint"));
    expect(schemes).toEqual(["dark", "light", "dark"]);
    page.destroy();
  });

  it("stop following once they are gone", () => {
    const reader = preference(false);
    const { schemes, mount } = recording();
    const page = createPage(place("light dark"), { components: { chart: mount } });
    page.render(tree);
    page.render({ type: "root", children: [] });
    reader.switchTo(true);
    expect(schemes).toEqual(["light"]);
    page.destroy();
  });

  it("leave a frame to follow the scheme its own document takes", () => {
    const reader = preference(false);
    const update = vi.fn();
    const page = createPage(place("light dark"), { components: { chart: rendersSlots(() => ({ update })) } });
    page.render(tree);
    reader.switchTo(true);
    expect(update).not.toHaveBeenCalled();
    page.destroy();
  });
});

describe("a region's components", () => {
  it("follow the scheme of the region's target", () => {
    preference(false);
    const { schemes, mount } = recording();
    const region = createRegion(place("light"), { components: { chart: mount } });
    region.render([chart]);
    root.style.colorScheme = "dark";
    region.scheme();
    expect(schemes).toEqual(["light", "dark"]);
    region.destroy();
  });
});

describe("a container that fails", () => {
  it("shows its content as its fallback, and gives the box back with its slots at the next update", async () => {
    preference(false);
    const card: ExpandedComponent = {
      type: "componentContainer",
      name: "card",
      params: [],
      children: [{ type: "paragraph", children: [{ type: "text", value: "Body" }] }],
      slots: [],
    };
    let draws = 0;
    const mount: Mount = (target, { slots, fail }) => {
      const draw = () => {
        draws += 1;
        target.replaceChildren(...(slots.default ?? []).map((item) => ("element" in item ? item.element : document.createElement("div"))));
        if (draws === 1) queueMicrotask(() => { fail("not drawn"); });
      };
      draw();
      return { update: draw };
    };
    const page = createPage(place("light"), { components: { card: mount } });
    page.render({ type: "root", children: [card] });
    await Promise.resolve();
    expect(root.querySelector(".compmd-component")?.innerHTML).toBe("<p>Body</p>");
    expect(root.querySelector(".compmd-item")).toBeNull();
    page.render({ type: "root", children: [card] });
    expect(root.querySelector(".compmd-item")?.textContent).toBe("Body");
    page.destroy();
  });
});
