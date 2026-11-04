// Where the page's DOM came from, on the fixtures' edits (trees a processor made, with origins): a page brought up to
// date by splices, whose kept blocks' origins are behind the text, maps as one given each whole tree does.
import { describe, expect, it, vi } from "vitest";
import { codePointOffset, utf16Offset } from "../src/keys.ts";
import { createPage, type Page } from "../src/page.ts";
import { components, renderer, scenarios, type Mode, type Render, type Scenario } from "./support.ts";

/** Each line the page gives, with what it measured for it: its text, or the element it was placed by. */
function linesOf(page: Page, elements: readonly Element[]): [number, string][] {
  let shown = "";
  const ranges = vi.spyOn(Range.prototype, "getClientRects").mockImplementation(function (this: Range) {
    shown = this.toString();
    return [{ x: 0, y: 0, width: 1, height: 1 }] as unknown as DOMRectList;
  });
  const boxes = vi.spyOn(Element.prototype, "getBoundingClientRect").mockImplementation(function (this: Element) {
    // The page's root too, for where its corner is: not a line's.
    if (elements.includes(this)) shown = `#${String(elements.indexOf(this))}`;
    return { x: 0, y: 0, width: 1, height: 1 } as DOMRect;
  });
  const lines: [number, string][] = [];
  for (const { line } of page.lines()) lines.push([line, shown]);
  ranges.mockRestore();
  boxes.mockRestore();
  return lines;
}

/**
 * After each render: the source of every element, the elements at every offset, by their place in the page, and each
 * line's place.
 */
function mappings(scenario: Scenario, mode: Mode): unknown[] {
  const root = document.body.appendChild(document.createElement("main"));
  const page: Page = createPage(root, { components: components([]) });
  const render = renderer(page, mode);
  const seen = scenario.renders.map((step) => {
    render(step);
    const elements = [...root.querySelectorAll("*")];
    return {
      sources: elements.map((element) => page.sourceOf(element)),
      at: Array.from({ length: codePointOffset(step.text, step.text.length) + 1 }, (_, offset) => page.elementsAt(offset).map((element) => elements.indexOf(element))),
      lines: linesOf(page, elements),
    };
  });
  page.destroy();
  root.remove();
  return seen;
}

const slice = (step: Render, start: number | undefined, end: number | undefined) => step.text.slice(utf16Offset(step.text, start ?? 0), utf16Offset(step.text, end ?? 0));

describe("sources on the fixtures' edits", () => {
  for (const scenario of [...scenarios("identity"), ...scenarios("invalid")]) {
    it(`${scenario.name}: splices map as whole trees do`, () => {
      expect(mappings(scenario, "splices")).toEqual(mappings(scenario, "trees"));
    });
  }

  it("gives each element the text it came from", () => {
    const [scenario] = scenarios("identity");
    const step = scenario!.renders[0]!;
    const root = document.body.appendChild(document.createElement("main"));
    const page = createPage(root, { components: components([]) });
    page.render(step.tree, { text: step.text });
    const source = page.sourceOf(root.querySelector("h1")!);
    expect(slice(step, source?.start.offset, source?.end.offset)).toMatch(/^# /);
    root.remove();
  });

  it("places each line by the text it holds", () => {
    const [scenario] = scenarios("identity");
    const step = scenario!.renders[0]!;
    const root = document.body.appendChild(document.createElement("main"));
    const page = createPage(root, { components: components([]) });
    page.render(step.tree, { text: step.text });
    const lines = linesOf(page, [...root.querySelectorAll("*")]);
    const written = step.text.split("\n");
    expect(lines.length).toBeGreaterThan(0);
    // A line's text, as the page shows it, is in the line as written (its markup and markers aside).
    for (const [line, shown] of lines) if (!shown.startsWith("#")) expect(written[line - 1]).toContain(shown.trim().split(/\s+/)[0]);
    root.remove();
  });

  it("gives a block that keeps its last good render its range in the current text", () => {
    const scenario = scenarios("invalid").find((s) => s.name.startsWith("Type an expression"))!;
    const root = document.body.appendChild(document.createElement("main"));
    const page = createPage(root, { components: components([]) });
    const render = renderer(page, "splices");
    const [first, halfWritten] = scenario.renders;
    render(first!);
    render(halfWritten!);
    const button = [...root.querySelectorAll("button")].find((b) => b.textContent === "A: 0")!;
    const source = page.sourceOf(button);
    expect(slice(halfWritten!, source?.start.offset, source?.end.offset)).toMatch(/^:: counter\[Counter A\] \{label=\(/);
    root.remove();
  });
});
