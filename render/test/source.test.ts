// Where the page's DOM came from, on the fixtures' edits (trees a processor made, with origins): a page brought up to
// date by splices, whose kept blocks' origins are behind the text, maps as one given each whole tree does.
import { describe, expect, it } from "vitest";
import { codePointOffset, utf16Offset } from "../src/keys.ts";
import { createPage, type Page } from "../src/page.ts";
import { components, renderer, scenarios, type Mode, type Render, type Scenario } from "./support.ts";

/** After each render: the source of every element, and the elements at every offset, by their place in the page. */
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
