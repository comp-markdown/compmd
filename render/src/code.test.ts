import { afterEach, beforeEach, describe, expect, it } from "vitest";
import type { Mount, MountContext, RenderError } from "./contract.ts";
import { createPage } from "./page.ts";
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
