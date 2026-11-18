// Invalid text mid-typing: each case turns the text invalid, as typing does halfway, and the
// next edit finishes it. With the rules for invalid text, every component keeps its instance and its state through
// both edits; lenient recovery alone (the same trees with no diagnostics) loses some in each case.
import { describe, expect, it } from "vitest";
import { createPage } from "../src/page.ts";
import { components, renderer, scenarios, type Mode, type Mounted, type Scenario } from "./support.ts";

interface Outcome {
  /** Components mounted again, or whose clicks were lost, after each edit. */
  broken: string[][];
  /** The page's text after each edit. */
  text: string[];
}

function run(scenario: Scenario, rules: boolean, mode: Mode = "trees"): Outcome {
  const root = document.body.appendChild(document.createElement("main"));
  const log: Mounted[] = [];
  const page = createPage(root, { components: components(log) });
  const render = renderer(page, mode, rules);
  const [first, ...steps] = scenario.renders;
  if (!first) throw new Error("no renders");
  render(first);
  for (const [i, button] of [...root.querySelectorAll("button")].entries()) for (let n = 0; n <= i; n++) button.click();
  const alive = log.filter((entry) => !entry.destroyed);
  const clicks = new Map(alive.map((entry) => [entry, entry.clicks()]));
  const outcome: Outcome = { broken: [], text: [] };
  for (const step of steps) {
    render(step);
    outcome.broken.push(alive.filter((entry) => entry.destroyed || entry.clicks() !== clicks.get(entry)).map((entry) => entry.label));
    outcome.text.push(root.textContent);
  }
  page.destroy();
  root.remove();
  return outcome;
}

const ALL = scenarios("invalid");
const named = (start: string) => ALL.find((s) => s.name.startsWith(start))!;

describe("invalid text mid-typing", () => {
  it("has the four cases", () => {
    expect(ALL).toHaveLength(4);
  });

  for (const scenario of ALL) {
    it(`${scenario.name}: the rules keep every component and its state`, () => {
      expect(run(scenario, true).broken).toEqual([[], []]);
    });

    it(`${scenario.name}: lenient recovery alone loses something`, () => {
      expect(run(scenario, false).broken.flat()).not.toEqual([]);
    });

    it(`${scenario.name}: splices give what whole trees give`, () => {
      expect(run(scenario, true, "splices")).toEqual(run(scenario, true));
    });
  }

  it("an unclosed container shows its opener as text, and the finished one renders", () => {
    const { text } = run(named("Type a new container"), true);
    expect(text[0]).toContain('::: card {title="New"}');
    expect(text[1]).not.toContain(":::");
    expect(text[1]).toContain("A new card.");
  });

  it("a half-written expression keeps the last good render, and the finished one updates the component", () => {
    const { text } = run(named("Type an expression"), true);
    expect(text[0]).toContain("A: 1");
    expect(text[1]).toContain("A2: 1");
  });

  it("broken frontmatter keeps the last good page, and the fixed one renders the new record", () => {
    const { text } = run(named("Edit the frontmatter"), true);
    expect(text[0]).not.toContain("Kiwis");
    expect(text[1]).toContain("Kiwis");
  });
});
