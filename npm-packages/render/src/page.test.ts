import { afterEach, describe, expect, it, vi } from "vitest";
import type { Mount, SlotInstance, SlotItem } from "./contract.ts";
import { createPage, type PageOptions } from "./page.ts";
import { inputLabel } from "./component-views.ts";
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

describe("mounting", () => {
  it("mounts each component once its box is in the page, under the page's theme, a slot's components after it", () => {
    const seen: { name: string; connected: boolean; accent: string | undefined }[] = [];
    // Each places its default slot's items in its box, as a component does.
    const recording =
      (name: string): Mount =>
      (target, { slots }) => {
        seen.push({ name, connected: target.isConnected, accent: target.closest<HTMLElement>(".compmd-page")?.style.getPropertyValue("--compmd-accent") });
        for (const item of (slots.default ?? []) as SlotItem[]) target.append(item.element);
      };
    const p = page({ components: { outer: recording("outer"), inner: recording("inner") }, tokens: { "--compmd-accent": "teal" } });
    p.render(tree(container("outer", [], [leaf("inner")])));
    expect(seen).toEqual([
      { name: "outer", connected: true, accent: "teal" },
      { name: "inner", connected: true, accent: "teal" },
    ]);
  });
});

describe("a container that fails on a later render", () => {
  it("shows its content as that render gives it, never as its first did", () => {
    let renders = 0;
    const panel: Mount = (target, { slots }) => {
      for (const item of (slots.default ?? []) as SlotItem[]) target.append(item.element);
      return {
        update: () => {
          renders += 1;
          throw new Error("can't draw these rows");
        },
      };
    };
    const field = leaf("text-input", [param("bind", "x", { input: "x" })], [text("Region")]);
    const at = (value: string): ExpandedRoot => ({ ...tree(container("panel", [], [field])), inputs: { x: { type: "string", value } } });
    const p = page({ components: { panel } });
    p.render(at("north"));
    p.render(at("south"));
    expect(renders).toBe(1);
    expect(root.querySelector("input")?.value).toBe("south");
  });
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
        { type: "list", ordered: true, start: 3, spread: true, children: [{ type: "listItem", children: [paragraph(text("three"))] }] },
        { type: "list", ordered: false, spread: false, children: [{ type: "listItem", checked: true, children: [paragraph(text("done"))] }] },
        { type: "blockquote", children: [paragraph(text("quoted"))] },
        { type: "thematicBreak" },
        { type: "code", lang: "js", value: "let a;" },
      ),
    );
    expect(root.innerHTML).toBe(
      "<h2>Title</h2>" +
        '<p><em>em</em><strong>strong</strong><del>del</del><code>code</code><br><a href="https://example.com" title="T">link</a><img src="a.png" alt="A"></p>' +
        '<ol start="3"><li><p>three</p></li></ol>' +
        '<ul class="compmd-tight"><li class="compmd-task"><input type="checkbox" disabled="" checked=""><p>done</p></li></ul>' +
        '<blockquote class="compmd-quote"><p>quoted</p></blockquote><hr>' +
        '<div class="compmd-code"><div class="compmd-code-bar" data-language="js" data-lines="1 line"><button type="button" class="compmd-code-copy" aria-label="Copy the code"></button></div>' +
        '<pre><code class="language-js"><span class="compmd-line">let a;</span></code></pre></div>',
    );
  });

  it("gives a heading its anchor as its id, and keeps it in step with the tree (spec §12.3)", () => {
    const anchored = (anchor: string, value: string): ExpandedNode => ({ ...heading(2, value), anchor });
    const shown = page();
    shown.render(tree(anchored("notes", "Notes"), anchored("café--bar", "Café & bar"), anchored("", "🎉")));
    expect(root.innerHTML).toBe('<h2 id="notes">Notes</h2><h2 id="café--bar">Café &amp; bar</h2><h2>🎉</h2>');
    shown.render(tree(anchored("notes", "Notes"), anchored("notes-1", "Notes")));
    expect([...root.querySelectorAll("h2")].map((element) => element.id)).toEqual(["notes", "notes-1"]);
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

  it("makes a reference a link once its definition is written, and text again once it is gone", () => {
    const reference = () => paragraph({ type: "linkReference", identifier: "d", referenceType: "full", children: [text("docs")] });
    const definition = { type: "definition", identifier: "d", url: "https://d.dev", title: null } as const;
    const p = page();
    p.render(tree(reference()));
    expect(root.innerHTML).toBe("<p><span>docs</span></p>");
    p.render(tree(reference(), definition));
    expect(root.innerHTML).toBe('<p><a href="https://d.dev">docs</a></p><template></template>');
    p.render(tree(reference()));
    expect(root.innerHTML).toBe("<p><span>docs</span></p>");
  });

  it("makes an image reference an image once its definition is written", () => {
    const reference = () => paragraph({ type: "imageReference", identifier: "logo", referenceType: "full", alt: "Logo" });
    const p = page();
    p.render(tree(reference()));
    expect(root.querySelector("p")?.innerHTML).toBe("<span>Logo</span>");
    p.render(tree(reference(), { type: "definition", identifier: "logo", url: "logo.png", title: null }));
    expect(root.querySelector("p")?.innerHTML).toBe('<img src="logo.png" alt="Logo">');
  });

  it("makes a wiki link a link once the host answers where it points, and text again when it no longer does", () => {
    let answer: string | undefined;
    const p = page({ wikiLink: () => answer });
    const nodes = () => tree(paragraph({ type: "wikiLink", target: "Home", label: "home" }));
    p.render(nodes());
    expect(root.innerHTML).toBe("<p><span>home</span></p>");
    answer = "/wiki/Home";
    p.render(nodes());
    expect(root.innerHTML).toBe('<p><a href="/wiki/Home">home</a></p>');
    answer = undefined;
    p.render(nodes());
    expect(root.innerHTML).toBe("<p><span>home</span></p>");
  });

  it("links with the host's own schemes, and shows other schemes' links as text", () => {
    const nodes = tree(
      paragraph(
        { type: "link", url: "app:item/7", title: null, children: [text("Books")] },
        { type: "linkReference", identifier: "trip", referenceType: "full", children: [text("Trip")] },
        { type: "wikiLink", target: "Home", label: "home" },
        { type: "link", url: "other:x", title: null, children: [text("Other")] },
      ),
      { type: "definition", identifier: "trip", url: "app:item/8", title: null },
    );
    const wikiLink = (target: string) => `app:page/${target}`;
    page({ wikiLink }).render(nodes);
    expect(root.querySelector("p")?.innerHTML).toBe("<span>Books</span><span>Trip</span><span>home</span><span>Other</span>");
    root.remove();
    page({ wikiLink, ownSchemes: ["app"] }).render(nodes);
    expect(root.querySelector("p")?.innerHTML).toBe('<a href="app:item/7">Books</a><a href="app:item/8">Trip</a><a href="app:page/Home">home</a><span>Other</span>');
  });

  it("links to and shows a pack's files where the host serves them, and without an answer their text and alt", () => {
    const place = "@acme/kit@1.4.2/icons/star.svg";
    const nodes = tree(paragraph({ type: "link", url: "@acme/kit@1.4.2/guide.comp.md", title: null, pack: true, children: [text("Guide")] }, { type: "image", url: place, alt: "Star", title: null, pack: true }));
    page().render(nodes);
    expect(root.querySelector("p")?.innerHTML).toBe('<span>Guide</span><img alt="Star">');
    root.remove();
    page({ packFile: (at) => (at === place ? undefined : `/compmd/packs/${at}`) }).render(nodes);
    expect(root.querySelector("p")?.innerHTML).toBe('<a href="/compmd/packs/@acme/kit@1.4.2/guide.comp.md">Guide</a><img alt="Star">');
    root.remove();
    page({ packFile: (at) => `/compmd/packs/${at}` }).render(nodes);
    expect(root.querySelector("img")?.getAttribute("src")).toBe(`/compmd/packs/${place}`);
  });

  it("shows images of the host's own schemes, in Markdown and raw HTML, and other schemes' images as their alt text", () => {
    const nodes = tree(
      paragraph(
        { type: "image", url: "app:file/3", alt: "Cover", title: null },
        { type: "imageReference", identifier: "map", referenceType: "full", alt: "Map" },
        { type: "html", value: '<img src="app:file/5" alt="Raw">' },
        { type: "image", url: "other:x", alt: "Other", title: null },
      ),
      { type: "definition", identifier: "map", url: "app:file/4", title: null },
    );
    const sources = () => [...root.querySelectorAll("img")].map((image) => [image.getAttribute("alt"), image.getAttribute("src")]);
    page().render(nodes);
    expect(sources()).toEqual([["Cover", null], ["Map", null], ["Raw", null], ["Other", null]]);
    root.remove();
    page({ ownSchemes: ["app"] }).render(nodes);
    expect(sources()).toEqual([["Cover", "app:file/3"], ["Map", "app:file/4"], ["Raw", "app:file/5"], ["Other", null]]);
  });

  it("asks its image source for images of the host's own schemes", async () => {
    const asked: string[] = [];
    const images = (src: string) => {
      asked.push(src);
      return Promise.resolve(`blob:${src}`);
    };
    page({ ownSchemes: ["app"], images }).render(tree(paragraph({ type: "image", url: "app:file/3", alt: "Cover", title: null }, { type: "image", url: "other:x", alt: "Other", title: null })));
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(asked).toEqual(["app:file/3"]);
    expect(root.querySelector("img")?.getAttribute("src")).toBe("blob:app:file/3");
  });

  it("refuses schemes a host can never name", () => {
    expect(() => page({ ownSchemes: ["javascript"] })).toThrow(/can't be a host's own scheme/);
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
    // A label, from the fallback before its colon: what follows is the default, which the value replaces.
    expect(select.closest("label")?.querySelector(".compmd-input-label")?.textContent).toBe("Region");
    const toggle = root.querySelector<HTMLInputElement>("input[type=checkbox]");
    expect(toggle?.checked).toBe(true);
    expect(toggle?.closest("label")?.textContent).toBe("");
    select.value = "all";
    select.dispatchEvent(new Event("change"));
    expect(onSet).toHaveBeenCalledWith("region", "all");
  });
});

describe("inputLabel", () => {
  it("is the fallback before its colon, or all of it", () => {
    expect(["From: today", "Show closed: no", "Search", " Region : all "].map(inputLabel)).toEqual(["From", "Show closed", "Search", "Region"]);
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

describe("splices", () => {
  /** A one-line block on `line`, at code points `start..end`. */
  const on = <T extends ExpandedNode>(node: T, line: number, start: number, end: number): T => ({
    ...node,
    origin: { start: { line, column: 1, offset: start }, end: { line, column: end - start + 1, offset: end }, occurrence: 0 },
  });
  const typed = (at: number, value: string) => ({ start: at, end: at, text: value });
  /** What a fresh page shows for `whole`. */
  const fresh = (whole: ExpandedRoot, options: PageOptions = {}) => {
    const other = document.createElement("main");
    createPage(other, options).render(whole);
    return other.innerHTML;
  };

  it("renders only the blocks a splice replaced, and the page is what the whole tree gives", () => {
    const shown = page();
    // "# A\n\none\n\ntwo\n"
    shown.render(tree(on(heading(1, "A"), 1, 0, 3), on(paragraph(text("one")), 3, 5, 8), on(paragraph(text("two")), 5, 10, 13)));
    const [h1, one, two] = [...root.children];
    shown.splice([{ from: 1, to: 2, nodes: [on(paragraph(text("one!")), 3, 5, 9)], lines: 0, offset: 1, edit: typed(8, "!") }]);
    expect(root.children[0]).toBe(h1);
    expect(root.children[1]).toBe(one);
    expect(root.children[2]).toBe(two);
    // "two" is at 11..14 now, though its origin, from before, says 10..13; an edit in it keeps its element.
    shown.splice([{ from: 2, to: 3, nodes: [on(paragraph(text("two?")), 5, 11, 15)], lines: 0, offset: 1, edit: typed(14, "?") }]);
    expect(root.children[2]).toBe(two);
    expect(root.innerHTML).toBe(fresh(tree(on(heading(1, "A"), 1, 0, 3), on(paragraph(text("one!")), 3, 5, 9), on(paragraph(text("two?")), 5, 11, 15))));
  });

  it("applies several splices in order, and blocks added and replaced in between", () => {
    const shown = page();
    shown.render(tree(on(paragraph(text("a")), 1, 0, 1), on(paragraph(text("b")), 3, 3, 4)));
    const b = root.children[1];
    shown.splice([
      { from: 0, to: 0, nodes: [on(paragraph(text("new")), 1, 0, 3)], lines: 2, offset: 5, edit: typed(0, "new\n\n") },
      { from: 0, to: 1, nodes: [on(paragraph(text("newer")), 1, 0, 5)], lines: 0, offset: 2, edit: typed(3, "er") },
    ]);
    expect(root.textContent).toBe("newerab");
    expect(root.children[2]).toBe(b);
  });

  it("numbers footnotes again when a splice elsewhere references one first", () => {
    const shown = page();
    const ref = (identifier: string) => ({ type: "footnoteReference", identifier });
    const definition = (identifier: string) => ({ type: "footnoteDefinition", identifier, children: [paragraph(text(`Note ${identifier}.`))] });
    shown.render(tree(on(paragraph(text("A"), ref("a")), 1, 0, 6), on(paragraph(text("B"), ref("b")), 3, 8, 14), on(definition("a"), 5, 16, 28), on(definition("b"), 6, 29, 41)));
    shown.splice([{ from: 0, to: 0, nodes: [on(paragraph(text("first"), ref("b")), 1, 0, 10)], lines: 2, offset: 12, edit: typed(0, "first[^b]\n\n") }]);
    expect([...root.querySelectorAll("sup a")].map((a) => a.textContent)).toEqual(["1", "2", "1"]);
    expect(root.innerHTML).toBe(
      fresh(tree(on(paragraph(text("first"), ref("b")), 1, 0, 10), on(paragraph(text("A"), ref("a")), 3, 12, 18), on(paragraph(text("B"), ref("b")), 5, 20, 26), on(definition("a"), 7, 28, 40), on(definition("b"), 8, 41, 53))),
    );
  });

  it("resolves link references in other blocks again when a definition changes", () => {
    const shown = page();
    const reference = paragraph({ type: "linkReference", identifier: "x", referenceType: "full", children: [text("X")] });
    shown.render(tree(on(reference, 1, 0, 6), on({ type: "definition", identifier: "x", url: "https://a.dev", title: null }, 3, 8, 26)));
    shown.splice([{ from: 1, to: 2, nodes: [on({ type: "definition", identifier: "x", url: "https://b.dev", title: null }, 3, 8, 26)], lines: 0, offset: 0, edit: { start: 21, end: 22, text: "b" } }]);
    expect(root.querySelector("a")?.getAttribute("href")).toBe("https://b.dev");
  });

  it("gives a new component the heading level of the last heading before it, in a block it didn't render", () => {
    const mount = vi.fn<Mount>(() => undefined);
    const shown = page({ components: { chart: mount } });
    shown.render(tree(on(heading(2, "Sales"), 1, 0, 8), on(paragraph(text("x")), 3, 10, 11)));
    shown.splice([{ from: 2, to: 2, nodes: [on(leaf("chart"), 5, 13, 21)], lines: 2, offset: 11, edit: typed(12, "\n:: chart[]") }]);
    expect(mount.mock.calls[0]?.[1].headingLevel).toBe(3);
  });

  it("renders raw HTML that pairs across blocks as one element, as the whole tree does", () => {
    const shown = page();
    const html = (value: string) => ({ type: "html", value });
    shown.render(tree(on(html("<details>"), 1, 0, 9), on(paragraph(text("inside")), 3, 11, 17), on(html("</details>"), 5, 19, 29), on(paragraph(text("after")), 7, 31, 36)));
    const after = root.children[1];
    shown.splice([{ from: 1, to: 2, nodes: [on(paragraph(text("inside!")), 3, 11, 18)], lines: 0, offset: 1, edit: typed(17, "!") }]);
    expect(root.innerHTML).toBe('<details><p>inside!</p></details><p data-line="7">after</p>');
    expect(root.children[1]).toBe(after);
    shown.splice([{ from: 2, to: 3, nodes: [], lines: -2, offset: -12, edit: { start: 19, end: 31, text: "" } }]);
    expect(root.innerHTML).toBe(fresh(tree(on(html("<details>"), 1, 0, 9), on(paragraph(text("inside!")), 3, 11, 18), on(paragraph(text("after")), 5, 19, 24))));
  });

  it("follows the inputs in blocks it didn't render, and the container breakpoints need", () => {
    const shown = page();
    const select = on(leaf("select-input", [param("bind", "r", { input: "r" })], [text("R")]), 1, 0, 20);
    shown.render({ type: "root", inputs: { r: { type: "enum", members: ["a", "b"], value: "a" } }, children: [select, on(paragraph(text("x")), 3, 22, 23)] });
    shown.splice([{ from: 1, to: 2, nodes: [on(container("box", [param("gap", "l", { breakpoint: "md" })], []), 3, 22, 40)], lines: 0, offset: 17, inputs: { r: { type: "enum", members: ["a", "b", "c"], value: "c" } } }]);
    expect(root.querySelector("select")?.value).toBe("c");
    expect(root.classList.contains("compmd-container")).toBe(true);
    shown.splice([{ from: 1, to: 2, nodes: [], lines: -2, offset: -19 }]);
    expect(root.classList.contains("compmd-container")).toBe(false);
  });

  it("keeps the last good page while the frontmatter doesn't parse, and follows every splice made meanwhile", () => {
    const shown = page();
    shown.render(tree(on(paragraph(text("a")), 1, 0, 1), on(paragraph(text("b")), 3, 3, 4)), { text: "a\n\nb\n" });
    const b = root.children[1];
    const broken = { text: "x", diagnostics: [{ code: "F001", line: 1, column: 1 }] };
    shown.splice([{ from: 0, to: 1, nodes: [on(paragraph(text("a2")), 1, 0, 2)], lines: 0, offset: 1, edit: typed(1, "2") }], broken);
    expect(root.textContent).toBe("ab");
    shown.splice([{ from: 1, to: 2, nodes: [on(paragraph(text("b3")), 3, 4, 6)], lines: 0, offset: 1, edit: typed(5, "3") }], { text: "a2\n\nb3\n" });
    expect(root.textContent).toBe("a2b3");
    expect(root.children[1]).toBe(b);
  });
});
