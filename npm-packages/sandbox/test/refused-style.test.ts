// The promise that inline styles apply in a component's box (spec §7.4), in host mode, in Chrome and WebKit: under a
// host page that allows them, nothing is reported; under one that refuses them (`style-src 'self'`), a trusted
// component's refused style is reported by the component's name, one a library writes in the page's body is reported
// with no name, and the frames keep the promise for strict and permissive components by their own policy.
import type { HostComponent } from "@compmd/processor";
import type { ExpandedRoot } from "@compmd/render";
import { afterEach, describe, expect, it } from "vitest";
import { createSandbox, type RefusedStyle, type View } from "../src/index.ts";
import { frames, pause, probe } from "./harness.ts";

/** A trusted component that styles its box inline, as most libraries that draw do. */
const styled = (target: Element) => {
  target.innerHTML = '<b style="color: rgb(1, 2, 3)">styled</b>';
};

/** A trusted component that measures in a temporary element in the page's body first, as Mermaid does. */
const measured = (target: Element) => {
  const measuring = document.body.appendChild(document.createElement("div"));
  measuring.setAttribute("style", "position: absolute; visibility: hidden");
  measuring.remove();
  target.textContent = "measured";
};

const trusted = (name: string): HostComponent => ({ name, language: "js", code: "", trust: { level: "trusted" } });
const tree = (name: string): ExpandedRoot => ({ type: "root", children: [{ type: "componentLeaf", name, params: [], fallback: [{ type: "text", value: name }] }] });

let view: View | undefined;
let area: HTMLElement | undefined;

afterEach(() => {
  view?.destroy();
  area?.remove();
});

/** The refusals reported while `name`, of `mounts`, draws in host mode. */
async function reported(name: string, mounts: Parameters<typeof createSandbox>[0]["trusted"]): Promise<RefusedStyle[]> {
  const refused: RefusedStyle[] = [];
  area = document.body.appendChild(document.createElement("div"));
  view = createSandbox({ base: frames, page: "host", ...(mounts ? { trusted: mounts } : {}) }).open(area, { components: [trusted(name)] });
  view.on("refusedStyle", (style) => refused.push(style));
  view.render(tree(name));
  await view.settled();
  // The browser reports a refusal in a task of its own.
  await pause(200);
  return refused;
}

/** The colours a `level` frame shows from a `<style>` element and from a `style` attribute. */
async function framed(level: "strict" | "permissive"): Promise<unknown> {
  const { frame, result } = probe(
    level,
    `async (target) => {
      target.innerHTML = '<style>.compmd-probe-sheet { color: rgb(4, 5, 6) }</style><b class="compmd-probe-sheet">sheet</b><i style="color: rgb(1, 2, 3)">attribute</i>';
      return { sheet: getComputedStyle(target.querySelector("b")).color, attribute: getComputedStyle(target.querySelector("i")).color };
    }`,
  );
  try {
    return await result;
  } finally {
    frame.destroy();
  }
}

// The policy is set once the first describe has run: a browser can't drop one a page has.
describe("a host page that allows inline styles", () => {
  it("reports nothing", async () => {
    expect(await reported("styled", { styled })).toEqual([]);
  });
});

describe("a host page that refuses inline styles", () => {
  it("is set", () => {
    const policy = document.head.appendChild(document.createElement("meta"));
    policy.httpEquiv = "Content-Security-Policy";
    policy.content = "style-src 'self'";
  });

  it("reports a trusted component's refused style by its name", async () => {
    const refused = await reported("styled", { styled });
    expect(refused.length).toBeGreaterThan(0);
    expect(refused.every((style) => style.component === "styled" && style.directive.startsWith("style-src"))).toBe(true);
  });

  it("reports a style refused outside every box, as a library that measures in the page's body writes, with no name", async () => {
    const refused = await reported("measured", { measured });
    expect(refused.length).toBeGreaterThan(0);
    expect(refused.every((style) => style.component === undefined)).toBe(true);
  });

  it("keeps the promise in strict and permissive frames, by their own policy", async () => {
    const expected = { sheet: "rgb(4, 5, 6)", attribute: "rgb(1, 2, 3)" };
    expect(await framed("strict")).toEqual(expected);
    expect(await framed("permissive")).toEqual(expected);
  }, 30_000);
});
