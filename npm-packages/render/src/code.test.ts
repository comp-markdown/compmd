import { afterEach, beforeEach, describe, expect, it } from "vitest";
import type { Mount, MountContext, RenderError } from "./contract.ts";
import { createPage } from "./page.ts";
import { createRegion } from "./region.ts";
import type { ExpandedNode } from "./tree.ts";

/** A Mermaid block a code renderer `mermaid` renders (spec §7.7), with its box's width. */
const rendered = (value: string, alt?: string): ExpandedNode => ({
  type: "code",
  lang: "mermaid",
  meta: null,
  value,
  component: {
    name: "mermaid",
    params: [
      { key: "code", breakpoint: null, value },
      { key: "width", breakpoint: null, value: "50%" },
    ],
  },
  ...(alt === undefined ? {} : { alt }),
});

let area: HTMLElement;
beforeEach(() => {
  area = document.body.appendChild(document.createElement("main"));
});
afterEach(() => {
  area.remove();
});

describe("a code block a code renderer renders", () => {
  it("mounts the renderer in its box, with the block's text as `code` and `alt` as the fallback text", () => {
    const mounts: MountContext[] = [];
    const updates: unknown[] = [];
    const mermaid: Mount = (target, context) => {
      mounts.push(context);
      target.textContent = `drawn: ${String(context.params.code)}`;
      return { update: (params) => updates.push(params.code) };
    };
    const hostsOwn = () => {
      throw new Error("the document's renderer wins over the host's");
    };
    const page = createPage(area, { components: { mermaid }, code: { mermaid: hostsOwn } });
    page.render({ type: "root", children: [rendered("A --> B", "A, then B")] });
    const box = area.querySelector(".compmd-component");
    expect(box?.textContent).toBe("drawn: A --> B");
    expect(mounts[0]?.params).toEqual({ code: "A --> B" });
    expect(mounts[0]?.fallback.text).toBe("A, then B");
    expect(mounts[0]?.fallback.element.querySelector("pre code.language-mermaid")?.textContent).toBe("A --> B");
    page.render({ type: "root", children: [rendered("A --> C", "A, then B")] });
    expect(updates).toEqual(["A --> C"]);
    expect(mounts).toHaveLength(1);
    expect(area.querySelector(".compmd-component")).toBe(box);
  });

  it("shows the code block when the renderer isn't here, and keeps it up to date", () => {
    const errors: RenderError[] = [];
    const page = createPage(area, { onError: (error) => errors.push(error) });
    page.render({ type: "root", children: [rendered("A --> B")] });
    const code = () => area.querySelector(".compmd-component pre code.language-mermaid")?.textContent;
    expect(code()).toBe("A --> B");
    expect(errors).toEqual([expect.objectContaining({ component: "mermaid", reason: "missing" })]);
    page.render({ type: "root", children: [rendered("A --> C")] });
    expect(code()).toBe("A --> C");
  });

  it("has its fallback's code block highlighted when the renderer isn't here", () => {
    const highlighted: string[] = [];
    const page = createPage(area, {
      highlight: (code, language) => {
        highlighted.push(`${language}: ${code}`);
        return undefined;
      },
    });
    page.render({ type: "root", children: [rendered("A --> B")] });
    expect(highlighted).toEqual(["mermaid: A --> B"]);
  });

  it("is a view of its own, apart from a plain block of the same language", () => {
    const mermaid: Mount = (target) => {
      target.append("drawn");
    };
    const page = createPage(area, { components: { mermaid } });
    page.render({ type: "root", children: [rendered("A --> B", "A, then B")] });
    page.render({ type: "root", children: [{ type: "code", lang: "mermaid", meta: null, value: "A --> B" }] });
    expect(area.querySelector(".compmd-component")).toBeNull();
    expect(area.querySelector("pre code.language-mermaid")?.textContent).toBe("A --> B");
  });
});

describe("a rendered block with `render=both`", () => {
  const both = (value: string): ExpandedNode => ({ ...rendered(value, "A, then B"), render: "both" });
  const mermaid: Mount = (target, context) => {
    target.textContent = `drawn: ${String(context.params.code)}`;
    return { update: (params) => (target.textContent = `drawn: ${String(params.code)}`) };
  };

  it("shows its code, then the drawing, in one box that takes the common params", () => {
    const page = createPage(area, { components: { mermaid } });
    page.render({ type: "root", children: [both("A --> B")] });
    const box = area.querySelector<HTMLElement>(".compmd-both");
    expect([...(box?.children ?? [])].map((child) => child.textContent)).toEqual(["A --> B", "drawn: A --> B"]);
    expect(getComputedStyle(box as HTMLElement).width).toBe("50%");
    expect(getComputedStyle(box?.querySelector(".compmd-component") as HTMLElement).width).not.toBe("50%");
    page.render({ type: "root", children: [both("A --> C")] });
    expect(area.querySelector(".compmd-both")).toBe(box);
    expect(box?.textContent).toBe("A --> Cdrawn: A --> C");
  });

  it("shows its code once when the renderer isn't here", () => {
    const page = createPage(area, { onError: () => undefined });
    page.render({ type: "root", children: [both("A --> B")] });
    expect(area.querySelectorAll("pre code.language-mermaid")).toHaveLength(1);
  });

  it("draws its component alone in a part, whose page shows the code: no code there, even when it can't run", () => {
    const part = createRegion(area, { part: true, components: { mermaid } });
    part.render([both("A --> B")]);
    expect(area.querySelector(".compmd-both")).toBeNull();
    expect(area.textContent).toBe("drawn: A --> B");
    part.stylesheet.flush();
    expect(part.stylesheet.text).not.toContain("50%");
    const missing = createRegion(area.appendChild(document.createElement("div")), { part: true, onError: () => undefined });
    missing.render([both("A --> B")]);
    expect(area.querySelectorAll("pre")).toHaveLength(0);
  });

  it("is a view apart from the same block drawn alone", () => {
    const page = createPage(area, { components: { mermaid } });
    page.render({ type: "root", children: [both("A --> B")] });
    page.render({ type: "root", children: [rendered("A --> B", "A, then B")] });
    expect(area.querySelector(".compmd-both")).toBeNull();
    expect(area.querySelector(".compmd-component")?.textContent).toBe("drawn: A --> B");
  });
});

describe("a code block that shows as code", () => {
  const block = (value: string, lang: string | null = "js"): ExpandedNode => ({ type: "code", lang, meta: null, value });

  it("is highlighted with the highlighter's tokens, in its lines, and again only when its text or language changes", () => {
    const calls: [string, string][] = [];
    const page = createPage(area, {
      highlight: (code, language) => {
        calls.push([code, language]);
        return [[{ text: code, bold: true }]];
      },
    });
    page.render({ type: "root", children: [block("let a;")] });
    expect(area.innerHTML).toBe('<pre><code class="language-js"><span class="compmd-line"><span style="font-weight: bold;">let a;</span></span></code></pre>');
    page.render({ type: "root", children: [block("let a;")], inputs: { x: { type: "string", value: "" } } });
    page.render({ type: "root", children: [block("let b;")] });
    page.render({ type: "root", children: [block("let b;", "ts")] });
    page.render({ type: "root", children: [block("plain", null)] });
    expect(calls).toEqual([
      ["let a;", "js"],
      ["let b;", "js"],
      ["let b;", "ts"],
      ["plain", ""],
    ]);
    expect(area.textContent).toBe("plain");
  });

  it("is plain without a highlighter", () => {
    createPage(area).render({ type: "root", children: [block("let a;")] });
    expect(area.innerHTML).toBe('<pre><code class="language-js"><span class="compmd-line">let a;</span></code></pre>');
  });
});
