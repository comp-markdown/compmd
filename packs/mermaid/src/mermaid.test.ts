// @vitest-environment happy-dom
// The component with Mermaid stubbed, since happy-dom can't lay a diagram out: which theme it draws with, what it shows
// when Mermaid can't read the source, and that the latest draw wins. The browser tests draw real diagrams.
import mermaid from "mermaid";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import mount, { isDark, themeOf } from "./mermaid.ts";

vi.mock("mermaid", () => ({ default: { initialize: vi.fn(), render: vi.fn() } }));

const render = vi.mocked(mermaid.render);
const initialize = vi.mocked(mermaid.initialize);

/** Mermaid's answer to `code`: an SVG that names it, or a parse error for `bad`. */
function answering(): void {
  render.mockImplementation((_id, code) =>
    code === "bad" ? Promise.reject(new Error("Parse error on line 1:\nbad\n^\nExpecting 'NEWLINE'")) : Promise.resolve({ svg: `<svg data-code="${code}"></svg>`, diagramType: "flowchart" }),
  );
}

/** The reader's preferred colour scheme, dark or not, for `matchMedia`. */
function scheme(dark: boolean): { change(dark: boolean): void } {
  const listeners = new Set<() => void>();
  const query = { matches: dark, addEventListener: (_: string, listener: () => void) => listeners.add(listener), removeEventListener: (_: string, listener: () => void) => listeners.delete(listener) };
  vi.stubGlobal("matchMedia", () => query);
  return {
    change: (next) => {
      query.matches = next;
      for (const listener of listeners) listener();
    },
  };
}

/** Every component a test mounted, destroyed after it. */
const mountedSoFar: { destroy(): void }[] = [];

/** Mounts the component on `code`, with `theme`. */
function mounted(code: string, theme: Parameters<typeof themeOf>[0] = "auto") {
  const target = document.createElement("div");
  const fallback = { text: "Draft, then review", element: Object.assign(document.createElement("pre"), { textContent: code }) };
  const component = mount(target, { params: { code, theme }, fallback });
  mountedSoFar.push(component);
  return { target, fallback, ...component };
}

const settle = () => new Promise((resolve) => setTimeout(resolve, 0));
const themes = () => initialize.mock.calls.map(([config]) => config.theme);

beforeEach(() => {
  vi.clearAllMocks();
  answering();
});

afterEach(() => {
  for (const component of mountedSoFar.splice(0)) component.destroy();
  document.documentElement.style.colorScheme = "";
});

describe("themeOf", () => {
  it("follows the reader with auto, and takes any other theme as given", () => {
    expect(themeOf("auto", false)).toBe("default");
    expect(themeOf("auto", true)).toBe("dark");
    expect(themeOf("forest", true)).toBe("forest");
  });
});

describe("mermaid", () => {
  it("draws the diagram Mermaid gives, labelled with the fallback text, in strict security", async () => {
    scheme(false);
    const { target } = mounted("graph LR; A --> B");
    await settle();
    const svg = target.querySelector("svg");
    expect(svg?.dataset.code).toBe("graph LR; A --> B");
    expect([svg?.getAttribute("role"), svg?.getAttribute("aria-label")]).toEqual(["img", "Draft, then review"]);
    expect(initialize).toHaveBeenCalledWith({ startOnLoad: false, securityLevel: "strict", suppressErrorRendering: true, theme: "default" });
  });

  it("draws again when the page's colour scheme changes, with auto only", async () => {
    scheme(false);
    mounted("graph LR; A --> B");
    mounted("graph LR; C --> D", "neutral");
    await settle();
    document.documentElement.style.colorScheme = "dark";
    await settle();
    expect(themes()).toEqual(["default", "neutral", "dark"]);
  });

  it("follows the reader's preference only where the page allows both schemes", async () => {
    const reader = scheme(false);
    document.documentElement.style.colorScheme = "light dark";
    mounted("graph LR; A --> B");
    await settle();
    reader.change(true);
    await settle();
    expect(themes()).toEqual(["default", "dark"]);
  });

  it("isn't dark for a reader who prefers dark on a page that sets no scheme", () => {
    expect(isDark({ matches: true })).toBe(false);
  });

  it("shows its source and what Mermaid found, when Mermaid can't read it", async () => {
    scheme(false);
    const { target, fallback } = mounted("bad");
    await settle();
    expect(target.firstChild).toBe(fallback.element);
    expect(target.querySelector("p")?.textContent).toBe("Mermaid couldn't read this diagram: Parse error on line 1:");
  });

  it("shows the latest params when an update overtakes a draw, and nothing once destroyed", async () => {
    scheme(false);
    const { target, update, destroy } = mounted("graph LR; A --> B");
    update({ code: "graph LR; B --> C", theme: "auto" });
    await settle();
    expect(target.querySelector("svg")?.dataset.code).toBe("graph LR; B --> C");
    update({ code: "graph LR; C --> D", theme: "auto" });
    destroy();
    await settle();
    expect(target.childNodes).toHaveLength(0);
  });
});
