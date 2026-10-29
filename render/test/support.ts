// Test support: the fixtures (edits of one document, as expanded trees with origins), and components with state kept in
// their instance, as a framework keeps it, so a remount loses it.
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import type { Mount, SlotInstance, SlotItem } from "../src/contract.ts";
import { codePointOffset, diff, utf16Offset } from "../src/keys.ts";
import type { Page } from "../src/page.ts";
import type { Diagnostic } from "../src/stable.ts";
import type { ExpandedNode, ExpandedRoot, Splice } from "../src/tree.ts";

export interface Render {
  text: string;
  tree: ExpandedRoot;
  diagnostics: Diagnostic[];
  edit?: { start: number; end: number };
}

/** One scenario: renders in order, the first `setup + 1` before the components are given state. */
export interface Scenario {
  name: string;
  setup: number;
  renders: Render[];
}

const FIXTURES = join(import.meta.dirname, "fixtures");

export function scenarios(folder: "identity" | "invalid"): Scenario[] {
  const dir = join(FIXTURES, folder);
  return readdirSync(dir)
    .filter((file) => file.endsWith(".json"))
    .map((file) => JSON.parse(readFileSync(join(dir, file), "utf8")) as Scenario);
}

/** `value` with every origin in it moved by `lines` and `offset`, as a splice moves the nodes after it. */
function moved(value: unknown, lines: number, offset: number): unknown {
  if (Array.isArray(value)) return value.map((item) => moved(item, lines, offset));
  if (value === null || typeof value !== "object") return value;
  return Object.fromEntries(
    Object.entries(value).map(([key, field]) => {
      if (key !== "origin") return [key, moved(field, lines, offset)];
      const { start, end, occurrence } = field as { start: { line: number; column: number; offset: number }; end: { line: number; column: number; offset: number }; occurrence: number };
      const point = (p: typeof start) => ({ ...p, line: p.line + lines, offset: p.offset + offset });
      return [key, { start: point(start), end: point(end), occurrence }];
    }),
  );
}

const lineCount = (text: string) => text.split("\n").length;
const same = (a: ExpandedNode | undefined, b: unknown) => a !== undefined && JSON.stringify(a) === JSON.stringify(b);

/**
 * The splice from `before` to `after`, as a processor that keeps the document open gives it: the top-level nodes
 * between those the two trees share, before (equal) and after (equal once moved by the edit), with the edit.
 */
export function spliceOf(before: Render, after: Render): Splice {
  const old = before.tree.children;
  const next = after.tree.children;
  const lines = lineCount(after.text) - lineCount(before.text);
  const offset = codePointOffset(after.text, after.text.length) - codePointOffset(before.text, before.text.length);
  let from = 0;
  while (from < Math.min(old.length, next.length) && same(old[from], next[from])) from++;
  let suffix = 0;
  while (suffix < Math.min(old.length, next.length) - from && same(next[next.length - 1 - suffix], moved(old[old.length - 1 - suffix], lines, offset))) suffix++;
  const { start, oldEnd, newEnd } = diff(before.text, after.text, after.edit);
  const inserted = after.text.slice(utf16Offset(after.text, start), utf16Offset(after.text, newEnd));
  return {
    from,
    to: old.length - suffix,
    nodes: next.slice(from, next.length - suffix),
    lines,
    offset,
    ...(after.tree.inputs ? { inputs: after.tree.inputs } : {}),
    ...(before.text === after.text ? {} : { edit: { start, end: oldEnd, text: inserted } }),
  };
}

/** How a test brings a page up to date: with whole trees, or with splices. */
export type Mode = "trees" | "splices";

/** A renderer for `page` that renders each of a scenario's renders after the first by `mode`. */
export function renderer(page: Page, mode: Mode, withDiagnostics = true): (render: Render) => void {
  let last: Render | undefined;
  return (render) => {
    const change = { text: render.text, ...(withDiagnostics ? { diagnostics: render.diagnostics } : {}) };
    if (mode === "splices" && last) page.splice([spliceOf(last, render)], change);
    else page.render(render.tree, { ...change, ...(render.edit ? { edit: render.edit } : {}) });
    last = render;
  };
}

/** A mounted component, as the test sees it. */
export interface Mounted {
  label: string;
  clicks: () => number;
  destroyed: boolean;
}

function placeItems(target: Element, items: readonly SlotItem[]): void {
  for (const item of items) if (item.element.parentNode !== target) target.append(item.element);
}

/** The identity document's components: counters, a notes field, a card with a slot, and a board with a template slot. */
export function components(log: Mounted[]): Record<string, Mount> {
  return {
    counter(target, { params }) {
      let clicks = 0;
      const button = target.appendChild(target.ownerDocument.createElement("button"));
      const entry: Mounted = { label: String(params.label), clicks: () => clicks, destroyed: false };
      log.push(entry);
      const show = () => {
        button.textContent = `${entry.label}: ${String(clicks)}`;
      };
      button.addEventListener("click", () => {
        clicks++;
        show();
      });
      show();
      return {
        update(next) {
          entry.label = String(next.label);
          show();
        },
        destroy() {
          entry.destroyed = true;
        },
      };
    },
    notes(target) {
      const area = target.appendChild(target.ownerDocument.createElement("textarea"));
      area.dataset.notes = "";
      const entry: Mounted = { label: "notes", clicks: () => 0, destroyed: false };
      log.push(entry);
      return {
        destroy() {
          entry.destroyed = true;
        },
      };
    },
    card(target, { slots }) {
      placeItems(target, (slots.default ?? []) as SlotItem[]);
      return {
        update(_params, next) {
          placeItems(target, (next.default ?? []) as SlotItem[]);
        },
      };
    },
    board(target, { slots }) {
      const place = (instances: SlotInstance[]) => {
        placeItems(target, instances.flatMap((instance) => instance.items));
      };
      place((slots.default ?? []) as SlotInstance[]);
      return {
        update(_params, next) {
          place((next.default ?? []) as SlotInstance[]);
        },
      };
    },
  };
}
