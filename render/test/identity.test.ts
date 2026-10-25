// The identity matrix: ten kinds of edit, on a document of components with state, rendered with keys from origins.
// Every component keeps its instance and its state, except where no key from the text can help: a record that moves
// in the data, and a block cut and pasted elsewhere.
import { describe, expect, it } from "vitest";
import { createPage } from "../src/page.ts";
import { components, scenarios, type Mounted, type Scenario } from "./support.ts";

interface Outcome {
  remounted: string[];
  /** Counters that show another counter's clicks. */
  moved: string[];
  /** Counters whose clicks are gone. */
  lost: string[];
  notesKept: boolean;
}

function run(scenario: Scenario): Outcome {
  const root = document.body.appendChild(document.createElement("main"));
  const log: Mounted[] = [];
  const page = createPage(root, {
    components: components(log),
    onError: ({ component, reason }) => {
      throw new Error(`${component} ${reason}`);
    },
  });
  const [first, ...rest] = scenario.renders;
  if (!first) throw new Error("no renders");
  page.render(first.tree, { text: first.text, diagnostics: first.diagnostics });
  const before = rest.slice(0, scenario.setup);
  const steps = rest.slice(scenario.setup);
  for (const render of before) page.render(render.tree, render);

  for (const [i, button] of [...root.querySelectorAll("button")].entries()) for (let n = 0; n <= i; n++) button.click();
  const clicks = new Map(log.filter((entry) => !entry.destroyed && entry.label !== "notes").map((entry) => [entry.label, entry.clicks()]));
  const area = root.querySelector<HTMLTextAreaElement>("textarea[data-notes]");
  if (area) {
    area.value = "typed";
    area.focus();
  }
  const alive = log.filter((entry) => !entry.destroyed);

  for (const render of steps) page.render(render.tree, render);

  const after = new Map(log.filter((entry) => !entry.destroyed).map((entry) => [entry.label, entry.clicks()]));
  const notes = root.querySelector<HTMLTextAreaElement>("textarea[data-notes]");
  const outcome = {
    remounted: alive.filter((entry) => entry.destroyed).map((entry) => entry.label),
    moved: [...clicks].filter(([label, count]) => (after.get(label) ?? 0) > 0 && after.get(label) !== count).map(([label]) => label),
    lost: [...clicks].filter(([label]) => after.get(label) === 0).map(([label]) => label),
    notesKept: notes?.value === "typed" && document.activeElement === notes,
  };
  page.destroy();
  root.remove();
  return outcome;
}

const ALL = scenarios("identity");
const KNOWN_LIMITS = new Set(["Prepend a record to the data", "Move a block (cut, then paste)"]);

describe("identity through edits, with keys from origins", () => {
  it("has the ten edits", () => {
    expect(ALL).toHaveLength(10);
  });

  for (const scenario of ALL.filter((s) => !KNOWN_LIMITS.has(s.name))) {
    it(`${scenario.name}: every component keeps its instance and its state`, () => {
      expect(run(scenario)).toEqual({ remounted: [], moved: [], lost: [], notesKept: true });
    });
  }

  it("prepending a record keeps the instances, but state stays with its place in the list (record keys are open, spec §17)", () => {
    const outcome = run(ALL.find((s) => s.name === "Prepend a record to the data")!);
    expect(outcome.remounted).toEqual([]);
    // Each counter keeps its place: the first, with Apples' clicks, now shows Kiwis, and the last one is new.
    expect(outcome.moved.sort()).toEqual(["Apples", "Pears", "board Apples", "board Pears"]);
    expect(outcome.lost.sort()).toEqual(["Plums", "board Plums"]);
  });

  it("a block cut and pasted elsewhere is mounted again, since cutting deletes its text", () => {
    expect(run(ALL.find((s) => s.name === "Move a block (cut, then paste)")!)).toEqual({ remounted: ["notes"], moved: [], lost: [], notesKept: false });
  });
});
