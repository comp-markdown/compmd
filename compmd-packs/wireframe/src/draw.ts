// A parsed wireframe as HTML (LANGUAGE.md, decision 5): each part drawn by a small function, the parts the pack
// doesn't draw yet as a box with their name and label, and the annotations as numbered notes beside the window.

import * as containers from "./containers.ts";
import * as controls from "./controls.ts";
import { attribute, childrenOf, element, textOf, type DrawPart, type Drawing } from "./dom.ts";
import { lookup } from "./lookup.ts";
import type { Part, Wireframe } from "./parse.ts";

/** The parts the pack draws (decision 4), each by its function; the others are drawn as a named box. */
export const DRAW: Readonly<Record<string, DrawPart>> = {
  window: containers.window,
  header: containers.header,
  footer: containers.footer,
  panel: containers.panel,
  section: containers.section,
  row: containers.row,
  col: containers.col,
  grid: containers.grid,
  cell: containers.cell,
  tabs: controls.tabs,
  tab: controls.tab,
  list: controls.list,
  item: controls.item,
  tree: controls.tree,
  node: controls.node,
  menubar: controls.menubar,
  menu: controls.menu,
  menuitem: controls.menuitem,
  separator: controls.divider,
  divider: controls.divider,
  text: controls.text,
  button: controls.button,
  input: controls.input,
  combo: controls.combo,
  checkbox: controls.checkbox,
  radio: controls.radio,
  toggle: controls.toggle,
  progress: controls.progress,
  image: controls.image,
  icon: controls.iconPart,
  spacer: controls.spacer,
};

/** A part the pack doesn't draw yet: a box with its name and label, holding its children. */
const named: DrawPart = (part, drawing) => {
  const label = textOf(part) ?? attribute(part, "label") ?? attribute(part, "name");
  return element(drawing.document, "div", ["named"], element(drawing.document, "span", ["name"], part.name), label === undefined ? undefined : element(drawing.document, "span", ["label"], label), ...childrenOf(part, drawing));
};

const SIDES = ["top", "left", "right", "bottom"] as const;

/** The annotations as numbered notes on their sides, each number also at its target. */
function annotate(document: Document, annotations: readonly Part[], targets: ReadonlyMap<string, HTMLElement>): HTMLElement[] {
  const sides = new Map(SIDES.map((side) => [side, element(document, "ol", ["notes", `notes-${side}`])]));
  for (const [i, note] of annotations.entries()) {
    const number = String(i + 1);
    const side = sides.get(attribute(note, "position") as (typeof SIDES)[number]) ?? sides.get("right");
    const item = element(document, "li", ["note"], element(document, "span", ["marker"], number), textOf(note) ?? "");
    side?.append(item);
    targets.get(attribute(note, "target") ?? "")?.append(element(document, "span", ["marker", "marker-at"], number));
  }
  return [...sides.values()].filter((side) => side.childElementCount > 0);
}

/**
 * `wireframe` drawn in `document`: the window, and its annotations beside it. The classes are `wf-…`, which
 * `STYLE` (style.ts) styles.
 *
 * @example
 * const { wireframe } = parse('window "Sign in":\n  button "Go" primary');
 * if (wireframe) target.append(draw(wireframe, document));
 */
export function draw(wireframe: Wireframe, document: Document): HTMLElement {
  const targets = new Map<string, HTMLElement>();
  const drawing: Drawing = {
    document,
    part: (part) => {
      const drawn = (lookup(DRAW, part.name) ?? named)(part, drawing);
      const id = attribute(part, "id");
      if (id !== undefined) targets.set(id, drawn);
      return drawn;
    },
  };
  const window = drawing.part(wireframe.window);
  return element(document, "div", ["wireframe"], window, ...annotate(document, wireframe.annotations, targets));
}
