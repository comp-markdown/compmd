// The component with Mermaid stubbed, since happy-dom can't lay a diagram out: which theme it draws with, and how it
// fails when Mermaid can't read the source. The browser tests draw real diagrams.
import { mountForTest } from "@compmd/pack/testing";
import mermaid from "mermaid";
import { beforeEach, describe, expect, it, vi } from "vitest";
import component, { themeOf, type Theme } from "./mermaid.ts";

vi.mock("mermaid", () => ({ default: { initialize: vi.fn(), render: vi.fn() } }));

const render = vi.mocked(mermaid.render);
const initialize = vi.mocked(mermaid.initialize);
const themes = () => initialize.mock.calls.map(([config]) => config.theme);

beforeEach(() => {
  vi.clearAllMocks();
  // Mermaid's answer to `code`: an SVG that names it, or a parse error for `bad`.
  render.mockImplementation((_id, code) =>
    code === "bad" ? Promise.reject(new Error("Parse error on line 1:\nbad\n^\nExpecting 'NEWLINE'")) : Promise.resolve({ svg: `<svg data-code="${code}"></svg>`, diagramType: "flowchart" }),
  );
});

/** The component drawing `code` with `theme`, in `scheme`. */
const drawing = (code: string, theme: Theme = "auto", scheme: "light" | "dark" = "light") => mountForTest(component, { params: { code, theme }, fallback: "Draft, then review", scheme });

describe("themeOf", () => {
  it("follows the page with auto, and takes any other theme as given", () => {
    expect(themeOf("auto", "light")).toBe("default");
    expect(themeOf("auto", "dark")).toBe("dark");
    expect(themeOf("forest", "dark")).toBe("forest");
  });
});

describe("mermaid", () => {
  it("draws the diagram Mermaid gives, as an image labelled with the fallback text, in strict security", async () => {
    const shown = drawing("graph LR; A --> B");
    await shown.settled();
    const image = shown.target.querySelector("[role=img]");
    expect(image?.getAttribute("aria-label")).toBe("Draft, then review");
    expect(image?.querySelector("svg")?.dataset.code).toBe("graph LR; A --> B");
    expect(initialize).toHaveBeenCalledWith({ startOnLoad: false, securityLevel: "strict", suppressErrorRendering: true, theme: "default" });
    shown.destroy();
  });

  it("draws in the page's scheme with auto, and again when it changes", async () => {
    const auto = drawing("graph LR; A --> B", "auto", "dark");
    const neutral = drawing("graph LR; C --> D", "neutral", "dark");
    await auto.settled();
    await neutral.settled();
    auto.setScheme("light");
    await auto.settled();
    expect(themes()).toEqual(["dark", "neutral", "default"]);
    auto.destroy();
    neutral.destroy();
  });

  it("fails with the first line of what Mermaid found, shows the code, and draws once the code is fixed", async () => {
    const shown = drawing("bad");
    await shown.settled();
    expect(shown.failures).toEqual(["Mermaid couldn't read this diagram: Parse error on line 1:"]);
    expect(shown.target.textContent).toBe("Draft, then review");
    shown.update({ code: "graph LR; A --> B", theme: "auto" });
    await shown.settled();
    expect(shown.target.querySelector("svg")?.dataset.code).toBe("graph LR; A --> B");
    shown.destroy();
  });
});
