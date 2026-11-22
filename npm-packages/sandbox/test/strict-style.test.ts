// What CompMD puts in a host's own page, under a policy that refuses inline styles (`style-src 'self'`, no
// `'unsafe-inline'`), as a host with a strict policy has: in host mode, math, highlighted code, boxes with params, an
// example and a framed component render, and the browser refuses nothing of theirs. In Chrome and WebKit.
import type { HostComponent } from "@compmd/processor";
import type { ExpandedNode, ExpandedRoot } from "@compmd/render";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { createSandbox, type View } from "../src/index.ts";
import { frames } from "./harness.ts";

/** The policy's refusals since it was set, each with its directive and what it refused. */
const refusals: string[] = [];
const record = (event: SecurityPolicyViolationEvent) => {
  refusals.push(`${event.effectiveDirective}: ${event.sample || event.blockedURI} (${event.target instanceof Element ? event.target.tagName.toLowerCase() : "document"})`);
};

let policy: HTMLMetaElement;
beforeAll(() => {
  document.addEventListener("securitypolicyviolation", record);
  policy = document.head.appendChild(document.createElement("meta"));
  policy.httpEquiv = "Content-Security-Policy";
  policy.content = "style-src 'self'";
});

afterAll(() => {
  document.removeEventListener("securitypolicyviolation", record);
});

const param = (key: string, value: unknown) => ({ key, breakpoint: null, value });
const text = (value: string): ExpandedNode => ({ type: "text", value });
const paragraph = (...children: ExpandedNode[]): ExpandedNode => ({ type: "paragraph", children });

/** A strict component that draws its text, in its frame. */
const greeting: HostComponent = { name: "greeting", language: "js", code: "export default (target) => { target.textContent = 'Hello'; };", trust: { level: "strict" } };

const tree: ExpandedRoot = {
  type: "root",
  children: [
    paragraph(text("The tide is "), { type: "inlineMath", value: "h = a \\cos(\\omega t)" }, text(".")),
    { type: "math", value: "\\sum_{n=1}^{12} n = 78" },
    { type: "code", lang: "ts", meta: null, value: "const tides = 2;\nexport default tides;" },
    {
      type: "componentContainer",
      name: "box",
      params: [param("display", "grid"), param("grid-template-columns", "1fr 1fr"), param("gap", "m"), param("padding", "s"), param("border", "subtle"), param("font-family", "mono"), param("color", "muted")],
      children: [paragraph(text("High water.")), paragraph(text("Low water."))],
      slots: [],
    },
    {
      type: "code",
      lang: "compmd",
      meta: "{render=both}",
      value: "::: box {background=subtle}\nAn example.\n::: /box",
      render: "both",
      document: {
        type: "root",
        example: 1,
        children: [{ type: "componentContainer", name: "box", params: [param("background", "subtle")], children: [paragraph(text("An example."))], slots: [] }],
      },
    },
    { type: "componentLeaf", name: "greeting", params: [param("width", "10rem")], fallback: [text("Hello")] },
  ],
};

describe("a page under a strict style-src, in host mode", () => {
  let view: View;
  let area: HTMLElement;

  afterAll(() => {
    view.destroy();
    area.remove();
  });

  it("renders math, highlighted code, boxes, an example and a component, with nothing refused", async () => {
    area = document.body.appendChild(document.createElement("div"));
    view = createSandbox({ base: frames, page: "host" }).open(area, { components: [greeting] });
    view.render(tree);
    await view.settled();
    // Math typeset, the code highlighted, the component drawn in its frame.
    await expect.poll(() => area.querySelectorAll("math").length, { timeout: 10_000 }).toBe(2);
    await expect.poll(() => area.querySelectorAll("pre code span[style], pre code .line span").length, { timeout: 10_000 }).toBeGreaterThan(0);
    expect(getComputedStyle(area.querySelector(".compmd-box")!).display).toBe("grid");
    expect(area.querySelector("iframe")).not.toBeNull();
    expect(refusals).toEqual([]);
  }, 30_000);

  it("is a policy the browser holds the page to: an inline style is refused", async () => {
    const probe = document.body.appendChild(document.createElement("div"));
    probe.innerHTML = '<b style="color: red">refused</b>';
    await expect.poll(() => refusals.length, { timeout: 5000 }).toBeGreaterThan(0);
    expect(getComputedStyle(probe.querySelector("b")!).color).not.toBe("rgb(255, 0, 0)");
    probe.remove();
  });
});
