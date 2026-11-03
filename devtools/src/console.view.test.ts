// The Console's input and what it shows of an evaluation: Enter evaluates, Shift+Enter adds a line, the arrow keys
// walk the history; a value as a tree, an F024 with a mark under its column.

import { flushSync, mount, unmount } from "svelte";
import { afterEach, describe, expect, it } from "vitest";
import type { Evaluation } from "./console.ts";
import ConsoleView from "./ConsoleView.svelte";

const targets: HTMLElement[] = [];
const mounted: ReturnType<typeof mount>[] = [];
afterEach(() => {
  for (const component of mounted.splice(0)) void unmount(component);
  for (const target of targets.splice(0)) target.remove();
});

/** The Console with `evaluations`, and what it asked to evaluate. */
function consoleWith(evaluations: Evaluation[] = []) {
  const target = document.body.appendChild(document.createElement("div"));
  targets.push(target);
  const asked: string[] = [];
  mounted.push(mount(ConsoleView, { target, props: { entries: [], evaluations, onevaluate: (expression: string) => asked.push(expression) } }));
  flushSync();
  const input = target.querySelector<HTMLTextAreaElement>("textarea.evaluate");
  if (!input) throw new Error("no input");
  /** Writes `text`, with the caret at its end. */
  const write = (text: string) => {
    input.value = text;
    input.setSelectionRange(text.length, text.length);
    input.dispatchEvent(new Event("input"));
    flushSync();
  };
  const press = (key: string, shiftKey = false) => {
    input.dispatchEvent(new KeyboardEvent("keydown", { key, shiftKey, bubbles: true, cancelable: true }));
    flushSync();
  };
  return { target, input, asked, write, press };
}

describe("the Console's input", () => {
  it("evaluates on Enter, adds a line on Shift+Enter, and walks the history with the arrow keys", () => {
    const { input, asked, write, press } = consoleWith();
    write("  md.doc  ");
    press("Enter");
    write("md.today()");
    press("Enter", true);
    expect(asked).toEqual(["md.doc"]);
    press("Enter");
    expect(asked).toEqual(["md.doc", "md.today()"]);
    expect(input.value).toBe("");
    write("half");
    press("ArrowUp");
    expect(input.value).toBe("md.today()");
    press("ArrowUp");
    expect(input.value).toBe("md.doc");
    press("ArrowDown");
    press("ArrowDown");
    expect(input.value).toBe("half");
  });

  it("doesn't evaluate what is only spaces", () => {
    const { asked, write, press } = consoleWith();
    write("   ");
    press("Enter");
    expect(asked).toEqual([]);
  });
});

describe("an evaluation in the Console", () => {
  it("shows the expression with its value as a tree, and an F024 with a mark under its column", () => {
    const { target } = consoleWith([
      { id: 1, at: 1, expression: "md.doc", result: { value: { title: "Sales" } } },
      { id: 2, at: 2, expression: "md.doc.tem", result: { error: { code: "F024", severity: "error", message: "unknown field 'tem'", range: { start: { line: 1, column: 8 } }, stage: "validate" } } },
      { id: 3, at: 3, expression: "md.today()" },
    ]);
    const evaluations = [...target.querySelectorAll("li.evaluation")];
    expect(evaluations[0]?.querySelector(".value")?.textContent).toContain("Sales");
    expect(evaluations[1]?.querySelector(".expression")?.textContent).toBe("md.doc.tem\n       ^");
    expect(evaluations[1]?.querySelector(".error")?.textContent).toBe("F024 unknown field 'tem'");
    expect(evaluations[2]?.textContent).toContain("Evaluating…");
  });
});
