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

  it("has its fallback's code block highlighted when the renderer isn't here", () => {
    const highlighted: string[] = [];
    const page = createPage(area, { highlight: (code, language) => highlighted.push(`${language}: ${code}`) });
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

describe("a code block that shows as code", () => {
  const block = (value: string, lang: string | null = "js"): ExpandedNode => ({ type: "code", lang, meta: null, value });

  it("is highlighted into its `<code>`, which holds its text until then, and again only when its text or language changes", () => {
    const calls: [string, string, string][] = [];
    const page = createPage(area, {
      highlight: (code, language, target) => {
        calls.push([code, language, target.textContent]);
        target.replaceChildren(Object.assign(document.createElement("span"), { textContent: code }));
      },
    });
    page.render({ type: "root", children: [block("let a;")] });
    expect(area.innerHTML).toBe('<pre><code class="language-js"><span>let a;</span></code></pre>');
    page.render({ type: "root", children: [block("let a;")], inputs: { x: { type: "string", value: "" } } });
    page.render({ type: "root", children: [block("let b;")] });
    page.render({ type: "root", children: [block("let b;", "ts")] });
    page.render({ type: "root", children: [block("plain", null)] });
    expect(calls).toEqual([
      ["let a;", "js", "let a;"],
      ["let b;", "js", "let b;"],
      ["let b;", "ts", "let b;"],
      ["plain", "", "plain"],
    ]);
    expect(area.innerHTML).toBe("<pre><code><span>plain</span></code></pre>");
  });

  it("is plain without a highlighter", () => {
    createPage(area).render({ type: "root", children: [block("let a;")] });
    expect(area.innerHTML).toBe('<pre><code class="language-js">let a;</code></pre>');
  });
});
