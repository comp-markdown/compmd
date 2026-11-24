import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Mount } from "./contract.ts";
import { createPage, type PageOptions } from "./page.ts";
import type { ExpandedComponent, ExpandedNode, ExpandedParam } from "./tree.ts";

const text = (value: string): ExpandedNode => ({ type: "text", value });
const paragraph = (...children: ExpandedNode[]): ExpandedNode => ({ type: "paragraph", children });
const param = (key: string, value: unknown, extra: Partial<ExpandedParam> = {}): ExpandedParam => ({ key, breakpoint: null, value, ...extra });
const leaf = (name: string, params: ExpandedParam[] = [], fallback: ExpandedNode[] = []): ExpandedComponent => ({ type: "componentLeaf", name, params, fallback });

/** A ```compmd block that renders `children` as example `number` (spec §7.8). */
const example = (number: number, children: ExpandedNode[], extra: Record<string, unknown> = {}): ExpandedNode => ({
  type: "code",
  lang: "compmd",
  meta: null,
  value: "The example's source.",
  render: "output",
  document: { type: "root", children, example: number },
  ...extra,
});

/** A component that writes `who` into its box. */
const writes = (who: string): Mount => (target) => {
  target.textContent = who;
};

let area: HTMLElement;
beforeEach(() => {
  area = document.body.appendChild(document.createElement("main"));
});
afterEach(() => {
  area.remove();
});

const page = (options: PageOptions = {}) => createPage(area, options);

describe("an example", () => {
  it("numbers its footnotes apart from the page's, with ids of its own", () => {
    const noted = (note: string): ExpandedNode[] => [
      paragraph(text("A claim"), { type: "footnoteReference", identifier: "n" }),
      { type: "footnoteDefinition", identifier: "n", children: [paragraph(text(note))] },
    ];
    page().render({ type: "root", children: [...noted("The page's note."), example(2, noted("The example's note."))] });
    const ids = [...area.querySelectorAll("[id]")].map((element) => element.id).sort();
    expect(ids).toEqual(["compmd-example-2-fn-1", "compmd-example-2-fnref-1", "compmd-fn-1", "compmd-fnref-1"]);
    expect(area.querySelector("#compmd-example-2-fn-1")?.textContent).toContain("The example's note.");
    expect(area.querySelector(".compmd-example a[href='#compmd-example-2-fn-1']")).not.toBeNull();
  });

  it("renders what its document renders, in a box with its params", () => {
    page().render({ type: "root", children: [example(1, [paragraph(text("Rated 4 of 5."))], { params: [param("padding", "s"), param("class", "demo")] })] });
    const box = area.querySelector(".compmd-example");
    expect(box?.classList.contains("demo")).toBe(true);
    expect(box?.querySelector(".compmd-example-document")?.textContent).toBe("Rated 4 of 5.");
    expect(box?.querySelector("pre")).toBeNull();
  });

  it("shows its code first with `render=both`", () => {
    page().render({ type: "root", children: [example(1, [paragraph(text("Drawn."))], { render: "both" })] });
    const box = area.querySelector(".compmd-example.compmd-both");
    expect([...(box?.children ?? [])].map((child) => child.textContent)).toEqual(["The example's source.", "", "Drawn."]);
  });

  it("runs its own components, never the document's of the same name", () => {
    page({ components: { hello: writes("the document's") }, examples: { 1: { hello: writes("the example's") } } }).render({
      type: "root",
      children: [leaf("hello", [], [text("Hi")]), example(1, [leaf("hello", [], [text("Hi")])]), example(2, [leaf("hello", [], [text("Hi")])])],
    });
    const drawn = [...area.querySelectorAll(".compmd-component:not(.compmd-example)")].map((box) => box.textContent);
    expect(drawn).toEqual(["the document's", "the example's", "Hi"]);
  });

  it("sets its own inputs, not the document's", () => {
    const onSet = vi.fn();
    const onSetExample = vi.fn();
    page({ onSet, onSetExample }).render({
      type: "root",
      children: [
        example(2, [leaf("toggle-input", [param("bind", "live", { input: "live" })])], {
          document: { type: "root", example: 2, inputs: { live: { type: "boolean", value: false } }, children: [leaf("toggle-input", [param("bind", "live", { input: "live" })])] },
        }),
      ],
    });
    const toggle = area.querySelector<HTMLInputElement>("input[type=checkbox]");
    expect(toggle?.checked).toBe(false);
    toggle?.click();
    expect(onSetExample).toHaveBeenCalledWith(2, "live", true);
    expect(onSet).not.toHaveBeenCalled();
  });

  it("follows a new render of its document, in the same box", () => {
    const shown = page();
    shown.render({ type: "root", children: [example(1, [paragraph(text("One."))])] });
    const box = area.querySelector(".compmd-example");
    shown.render({ type: "root", children: [example(1, [paragraph(text("Two."))])] });
    expect(area.querySelector(".compmd-example")).toBe(box);
    expect(box?.textContent).toBe("Two.");
  });
});
