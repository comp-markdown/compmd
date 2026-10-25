import { afterEach, describe, expect, it, vi } from "vitest";
import type { Mount, SlotInstance, SlotItem } from "./contract.ts";
import { createPage, type PageOptions } from "./page.ts";
import type { ExpandedComponent, ExpandedNode, ExpandedParam, ExpandedRoot } from "./tree.ts";

const text = (value: string): ExpandedNode => ({ type: "text", value });
const paragraph = (...children: ExpandedNode[]): ExpandedNode => ({ type: "paragraph", children });
const heading = (depth: number, value: string): ExpandedNode => ({ type: "heading", depth, children: [text(value)] });
const param = (key: string, value: unknown, extra: Partial<ExpandedParam> = {}): ExpandedParam => ({ key, breakpoint: null, value, ...extra });
const leaf = (name: string, params: ExpandedParam[] = [], fallback: ExpandedNode[] = []): ExpandedComponent => ({ type: "componentLeaf", name, params, fallback });
const container = (name: string, params: ExpandedParam[], children: ExpandedNode[], extra: Partial<ExpandedComponent> = {}): ExpandedComponent => ({
  type: "componentContainer",
  name,
  params,
  children,
  slots: [],
  ...extra,
});
const tree = (...children: ExpandedNode[]): ExpandedRoot => ({ type: "root", children });

let root: HTMLElement;
function page(options: PageOptions = {}) {
  root = document.body.appendChild(document.createElement("main"));
  return createPage(root, options);
}
afterEach(() => {
  root.remove();
});

describe("Markdown", () => {
  it("renders CommonMark and GFM", () => {
    page().render(
      tree(
        heading(2, "Title"),
        paragraph(
          { type: "emphasis", children: [text("em")] },
          { type: "strong", children: [text("strong")] },
          { type: "delete", children: [text("del")] },
          { type: "inlineCode", value: "code" },
          { type: "break" },
          { type: "link", url: "https://example.com", title: "T", children: [text("link")] },
          { type: "image", url: "a.png", alt: "A" },
        ),
        { type: "list", ordered: true, start: 3, children: [{ type: "listItem", children: [paragraph(text("three"))] }] },
        { type: "list", ordered: false, children: [{ type: "listItem", checked: true, children: [paragraph(text("done"))] }] },
        { type: "blockquote", children: [paragraph(text("quoted"))] },
        { type: "thematicBreak" },
        { type: "code", lang: "js", value: "let a;" },
      ),
    );
    expect(root.innerHTML).toBe(
      "<h2>Title</h2>" +
        '<p><em>em</em><strong>strong</strong><del>del</del><code>code</code><br><a href="https://example.com" title="T">link</a><img src="a.png" alt="A"></p>' +
        '<ol start="3"><li><p>three</p></li></ol>' +
        '<ul><li class="compmd-task"><input type="checkbox" disabled="" checked=""><p>done</p></li></ul>' +
        "<blockquote><p>quoted</p></blockquote><hr>" +
        '<pre><code class="language-js">let a;</code></pre>',
    );
  });

  it("renders a table with its header row and alignment", () => {
    const row = (...cells: string[]): ExpandedNode => ({ type: "tableRow", children: cells.map((cell) => ({ type: "tableCell", children: [text(cell)] })) });
    page().render(tree({ type: "table", align: [null, "right"], children: [row("A", "B"), row("1", "2")] }));
    expect(root.innerHTML).toBe('<table><thead><tr><th>A</th><th class="compmd-align-right">B</th></tr></thead><tbody><tr><td>1</td><td class="compmd-align-right">2</td></tr></tbody></table>');
  });

  it("renders an alert with its title (spec §2.3)", () => {
    page().render(tree({ type: "blockquote", alert: "warning", children: [paragraph(text("Careful."))] }));
    expect(root.innerHTML).toBe('<div class="compmd-alert compmd-alert-warning" role="note"><p class="compmd-alert-title">Warning</p><p>Careful.</p></div>');
  });

  it("shows math as its TeX source without a math renderer, and through one when given", () => {
    const math = tree(paragraph({ type: "inlineMath", value: "x^2" }), { type: "math", value: "\\sum x" });
    page().render(math);
    expect(root.innerHTML).toBe('<p><code class="language-math math-inline">x^2</code></p><pre><code class="language-math math-display">\\sum x</code></pre>');
    root.remove();
    page({ math: (tex, { display }, target) => (target.textContent = `${display ? "D" : "I"}(${tex})`) }).render(math);
    expect(root.textContent).toBe("I(x^2)D(\\sum x)");
  });

  it("renders a code block through the caller's renderer for its language, or its source if that throws", () => {
    page({ code: { mermaid: (source, target) => (target.textContent = `diagram: ${source}`), bad: () => { throw new Error("no"); } } }).render(
      tree({ type: "code", lang: "mermaid", value: "graph TD" }, { type: "code", lang: "bad", value: "src" }),
    );
    expect(root.innerHTML).toBe('<div class="compmd-code language-mermaid">diagram: graph TD</div><div class="compmd-code language-bad">src</div>');
  });

  it("renders raw HTML through the allowlist, and drops unsafe links", () => {
    page().render(tree(paragraph({ type: "html", value: "<kbd>" }, text("Ctrl"), { type: "html", value: "</kbd>" }, { type: "html", value: "<script>" }), paragraph({ type: "link", url: "javascript:x", children: [text("x")] })));
    expect(root.innerHTML).toBe("<p><kbd>Ctrl</kbd>&lt;script&gt;</p><p><span>x</span></p>");
  });

  it("numbers footnotes in reading order and lists them at the end", () => {
    page().render(
      tree(
        paragraph(text("A"), { type: "footnoteReference", identifier: "b" }, text(" C"), { type: "footnoteReference", identifier: "a" }),
        { type: "footnoteDefinition", identifier: "a", children: [paragraph(text("Note a."))] },
        { type: "footnoteDefinition", identifier: "b", children: [paragraph(text("Note b."))] },
      ),
    );
    expect([...root.querySelectorAll("sup a")].map((a) => a.textContent)).toEqual(["1", "2"]);
    expect([...root.querySelectorAll(".compmd-footnotes li")].map((li) => [li.id, li.querySelector("p")?.textContent])).toEqual([
      ["compmd-fn-1", "Note b."],
      ["compmd-fn-2", "Note a."],
    ]);
  });

  it("resolves link references, and wiki links when the caller says where they point", () => {
    const nodes = tree(
      paragraph({ type: "linkReference", identifier: "docs", referenceType: "full", children: [text("Docs")] }, { type: "wikiLink", target: "Home", label: "home" }),
      { type: "definition", identifier: "docs", url: "https://d.dev", title: null },
    );
    page().render(nodes);
    expect(root.querySelector("p")?.innerHTML).toBe('<a href="https://d.dev">Docs</a><span>home</span>');
    root.remove();
    page({ wikiLink: (target) => `/wiki/${target}` }).render(nodes);
    expect(root.querySelector("p")?.innerHTML).toBe('<a href="https://d.dev">Docs</a><a href="/wiki/Home">home</a>');
  });
});

describe("the page", () => {
  it("is the container breakpoints query, with the theme tokens", () => {
    const shown = page({ tokens: { "--compmd-space-m": "1.5rem", color: "red" } });
    expect(root.classList.contains("compmd-page")).toBe(true);
    expect(root.style.getPropertyValue("--compmd-space-m")).toBe("1.5rem");
    expect(root.style.getPropertyValue("color")).toBe("");
    shown.theme({ "--compmd-accent": "teal" });
    expect(root.style.getPropertyValue("--compmd-space-m")).toBe("");
    expect(root.style.getPropertyValue("--compmd-accent")).toBe("teal");
  });

  it("makes a box the container of blocks in it that use breakpoints, and a column when one fills it", () => {
    page().render(tree(container("box", [], [container("box", [param("gap", "l", { breakpoint: "md" })], [paragraph(text("x"))])]), container("box", [], [container("box", [param("fill", true)], [])])));
    const [outer, filled] = [...root.children] as HTMLElement[];
    expect(outer?.classList.contains("compmd-container")).toBe(true);
    expect(outer?.firstElementChild?.classList.contains("compmd-container")).toBe(false);
    expect(filled?.classList.contains("compmd-fills")).toBe(true);
  });

  it("renders input components from the root's inputs, and sets the input when the reader changes one", () => {
    const onSet = vi.fn();
    const inputs = { region: { type: "enum", members: ["all", "eu"], value: "eu" }, live: { type: "boolean", value: true } };
    page({ onSet }).render({
      type: "root",
      inputs,
      children: [leaf("select-input", [param("bind", "region", { input: "region" })], [text("Region: all")]), leaf("toggle-input", [param("bind", "live", { input: "live" })])],
    });
    const select = root.querySelector("select")!;
    expect([...select.options].map((o) => o.value)).toEqual(["all", "eu"]);
    expect(select.value).toBe("eu");
    expect(select.getAttribute("aria-label")).toBe("Region: all");
    expect(root.querySelector<HTMLInputElement>("input[type=checkbox]")?.checked).toBe(true);
    select.value = "all";
    select.dispatchEvent(new Event("change"));
    expect(onSet).toHaveBeenCalledWith("region", "all");
  });
});

describe("host-language components", () => {
  it("gives a component its own params, its heading level, and its box the common params", () => {
    const mount = vi.fn<Mount>(() => undefined);
    page({ components: { chart: mount } }).render(tree(heading(2, "Sales"), leaf("chart", [param("rows", [1, 2]), param("padding", "m"), param("class", ["wide"])], [text("Sales chart")])));
    const [target, context] = mount.mock.calls[0]!;
    expect(context).toMatchObject({ params: { rows: [1, 2] }, headingLevel: 3, fallback: { text: "Sales chart" } });
    expect(target.classList.contains("compmd-component")).toBe(true);
    expect(target.classList.contains("wide")).toBe(true);
  });

  it("updates a component in place rather than mounting it again", () => {
    const update = vi.fn();
    const mount = vi.fn<Mount>(() => ({ update }));
    const shown = page({ components: { chart: mount } });
    shown.render(tree(leaf("chart", [param("rows", [1])])));
    shown.render(tree(leaf("chart", [param("rows", [1, 2])])));
    expect(mount).toHaveBeenCalledTimes(1);
    expect(update).toHaveBeenCalledWith({ rows: [1, 2] }, {});
  });

  it("renders slots up front as items, with each component's name and params", () => {
    let slots: Record<string, SlotItem[] | SlotInstance[]> = {};
    const tabs: Mount = (_target, context) => {
      slots = context.slots;
    };
    page({ components: { tabs, tab: () => undefined } }).render(
      tree(container("tabs", [], [container("tab", [param("label", "One")], [paragraph(text("1"))]), paragraph(text("note"))], { slots: [{ type: "componentSlot", name: "footer", children: [paragraph(text("f"))] }] })),
    );
    const items = slots.default as SlotItem[];
    expect(items.map((item) => item.component)).toEqual([{ name: "tab", params: { label: "One" } }, undefined]);
    expect(items.every((item) => item.element.classList.contains("compmd-item"))).toBe(true);
    expect((slots.footer as SlotItem[])[0]?.element.textContent).toBe("f");
  });

  it("gives a template slot's instances with their values", () => {
    let slots: Record<string, SlotItem[] | SlotInstance[]> = {};
    page({ components: { board: (_t, context) => void (slots = context.slots) } }).render(
      tree(container("board", [], [], { instances: [{ value: "a", children: [paragraph(text("A"))] }, { value: "b", children: [paragraph(text("B"))] }] })),
    );
    expect((slots.default as SlotInstance[]).map((instance) => [instance.value, instance.items[0]?.element.textContent])).toEqual([
      ["a", "A"],
      ["b", "B"],
    ]);
  });

  it("lets a component set the input bound to a param, and refuses any other param and during update", () => {
    const onSet = vi.fn();
    let set: (param: string, value: unknown) => void = () => undefined;
    const errors: unknown[] = [];
    const mount: Mount = (_target, context) => {
      set = context.set;
      return {
        update() {
          try {
            context.set("selected", "x");
          } catch (error) {
            errors.push(error);
          }
        },
      };
    };
    const shown = page({ components: { chart: mount }, onSet });
    const node = leaf("chart", [param("selected", "eu", { input: "region" }), param("rows", [])]);
    shown.render(tree(node));
    set("selected", "us");
    expect(onSet).toHaveBeenCalledWith("region", "us");
    expect(() => {
      set("rows", []);
    }).toThrow(/isn't bound to an input/);
    shown.render(tree(leaf("chart", [param("selected", "us", { input: "region" }), param("rows", [])])));
    expect(String(errors[0])).toMatch(/refused during update/);
  });

  it("shows the fallback of a component that is missing or throws (W118)", () => {
    const onError = vi.fn();
    page({ components: { broken: () => { throw new Error("boom"); } }, onError }).render(
      tree(leaf("missing", [], [text("Missing chart")]), container("broken", [], [paragraph(text("Inside"))])),
    );
    expect(root.textContent).toBe("Missing chartInside");
    expect(onError.mock.calls.map(([error]) => [(error as { component: string }).component, (error as { reason: string }).reason])).toEqual([
      ["missing", "missing"],
      ["broken", "threw"],
    ]);
  });

  it("destroys components that are gone", () => {
    const destroy = vi.fn();
    const shown = page({ components: { chart: () => ({ destroy }) } });
    shown.render(tree(leaf("chart")));
    shown.render(tree(paragraph(text("no chart"))));
    expect(destroy).toHaveBeenCalledTimes(1);
    expect(root.querySelector(".compmd-component")).toBeNull();
  });
});
