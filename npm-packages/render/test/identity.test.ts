// The identity matrix: ten kinds of edit, on a document of components with state, rendered with keys from origins,
// as whole trees and as splices. Every component keeps its instance and its state, except where no key from the text
// can help: a record that moves in the data, and a block cut and pasted elsewhere.
import { describe, expect, it } from "vitest";
import { createPage } from "../src/page.ts";
import { components, renderer, scenarios, spliceOf, type Mode, type Mounted, type Scenario } from "./support.ts";

interface Outcome {
  remounted: string[];
  /** Counters that show another counter's clicks. */
  moved: string[];
  /** Counters whose clicks are gone. */
  lost: string[];
  notesKept: boolean;
  /** The page at the end. */
  html: string;
}

function run(scenario: Scenario, mode: Mode): Outcome {
  const root = document.body.appendChild(document.createElement("main"));
  const log: Mounted[] = [];
  const page = createPage(root, {
    components: components(log),
    onError: (error) => {
      throw new Error(JSON.stringify(error));
    },
  });
  const render = renderer(page, mode);
  const [first, ...rest] = scenario.renders;
  if (!first) throw new Error("no renders");
  render(first);
  const before = rest.slice(0, scenario.setup);
  const steps = rest.slice(scenario.setup);
  for (const each of before) render(each);

  for (const [i, button] of [...root.querySelectorAll("button")].entries()) for (let n = 0; n <= i; n++) button.click();
  const clicks = new Map(log.filter((entry) => !entry.destroyed && entry.label !== "notes").map((entry) => [entry.label, entry.clicks()]));
  const area = root.querySelector<HTMLTextAreaElement>("textarea[data-notes]");
  if (area) {
    area.value = "typed";
    area.focus();
  }
  const alive = log.filter((entry) => !entry.destroyed);

  for (const each of steps) render(each);

  const after = new Map(log.filter((entry) => !entry.destroyed).map((entry) => [entry.label, entry.clicks()]));
  const notes = root.querySelector<HTMLTextAreaElement>("textarea[data-notes]");
  const outcome = {
    remounted: alive.filter((entry) => entry.destroyed).map((entry) => entry.label),
    moved: [...clicks].filter(([label, count]) => (after.get(label) ?? 0) > 0 && after.get(label) !== count).map(([label]) => label),
    lost: [...clicks].filter(([label]) => after.get(label) === 0).map(([label]) => label),
    notesKept: notes?.value === "typed" && document.activeElement === notes,
    html: root.innerHTML,
  };
  page.destroy();
  root.remove();
  return outcome;
}

const ALL = scenarios("identity");
const KNOWN_LIMITS = new Set(["Prepend a record to the data", "Move a block (cut, then paste)"]);

const named = (name: string) => ALL.find((s) => s.name === name)!;

describe("identity through edits, with keys from origins", () => {
  it("has the ten edits", () => {
    expect(ALL).toHaveLength(10);
  });

  for (const scenario of ALL.filter((s) => !KNOWN_LIMITS.has(s.name))) {
    it(`${scenario.name}: every component keeps its instance and its state`, () => {
      expect(run(scenario, "trees")).toEqual({ remounted: [], moved: [], lost: [], notesKept: true, html: expect.any(String) as string });
    });
  }

  it("splices the blocks an edit touched", () => {
    const [first, second] = named("Type above a component").renders;
    expect(spliceOf(first!, second!)).toMatchObject({ from: 1, to: 2, lines: 0, offset: 7, edit: { start: 69, end: 69 } });
  });

  for (const scenario of ALL) {
    it(`${scenario.name}: splices give what whole trees give, page and all`, () => {
      expect(run(scenario, "splices")).toEqual(run(scenario, "trees"));
    });
  }

  it("prepending a record keeps the instances, but state stays with its place in the list (record keys are open, spec §17)", () => {
    const outcome = run(named("Prepend a record to the data"), "trees");
    expect(outcome.remounted).toEqual([]);
    // Each counter keeps its place: the first, with Apples' clicks, now shows Kiwis, and the last one is new.
    expect(outcome.moved.sort()).toEqual(["Apples", "Pears", "board Apples", "board Pears"]);
    expect(outcome.lost.sort()).toEqual(["Plums", "board Plums"]);
  });

  it("a block cut and pasted elsewhere is mounted again, since cutting deletes its text", () => {
    expect(run(named("Move a block (cut, then paste)"), "trees")).toMatchObject({ remounted: ["notes"], moved: [], lost: [], notesKept: false });
  });
});
