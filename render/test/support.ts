// Test support: the fixtures (edits of one document, as expanded trees with origins), and components with state kept in
// their instance, as a framework keeps it, so a remount loses it.
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import type { Mount, SlotInstance, SlotItem } from "../src/contract.ts";
import type { Diagnostic } from "../src/stable.ts";
import type { ExpandedRoot } from "../src/tree.ts";

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
