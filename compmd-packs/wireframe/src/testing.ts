// What the drawing's tests share: a part read from one line, a drawing that marks each child it is asked for, and
// LANGUAGE.md's examples.

import type { Drawing } from "./dom.ts";
import { draw } from "./draw.ts";
import { parse, type Part } from "./parse.ts";

/**
 * The part written as `lines`, its children indented below it, as a window's first child.
 *
 * @throws Error naming the source's errors, when it isn't valid.
 */
export function partOf(...lines: string[]): Part {
  const { wireframe, errors } = parse(["window:", ...lines.map((line) => `  ${line}`)].join("\n"));
  const part = wireframe?.window.children[0];
  if (!part) throw new Error(errors.map(String).join("\n"));
  return part;
}

/**
 * The first child of the part written as `parent`, its own line, and `lines` below it, for a part that is only in
 * one parent: `childOf("tabs:", 'tab "General"')`.
 */
export function childOf(parent: string, ...lines: string[]): Part {
  const part = partOf(parent, ...lines.map((line) => `  ${line}`)).children[0];
  if (!part) throw new Error(`\`${parent}\` has no child`);
  return part;
}

/** A drawing whose children are each a `<i>` holding their name, so a part's function is tested on its own. */
export const drawing: Drawing = {
  document,
  part: (part) => {
    const child = document.createElement("i");
    child.textContent = part.name;
    return child;
  },
};

/** The texts of what `selector` finds in `root`. */
export const texts = (root: Element, selector: string) => [...root.querySelectorAll(selector)].map((found) => found.textContent);
// LANGUAGE.md's examples.
export const SETTINGS = `window "Settings":
  tabs:
    tab "General" active
    tab "Privacy"
    tab "Advanced"
  section "Notifications":
    toggle "Email me about replies" on
    checkbox "Weekly summary" checked
  section "Theme":
    radio "Light" group="theme"
    radio "Dark" group="theme" selected
    radio "Follow the system" group="theme"
  footer:
    row justify=end:
      button "Cancel"
      button "Save" primary`;

export const FILES = `window "Files":
  menubar:
    menu "File":
      menuitem "New folder" shortcut="⌘N"
      separator
      menuitem "Close" shortcut="⌘W"
    menu "Edit"
  row:
    col 220:
      tree:
        node "Documents" selected:
          node "Reports"
          node "Drafts" collapsed
        node "Pictures"
    col fill:
      grid cols=3 rows=2:
        cell "report.pdf" row=1 col=1
        cell "notes.txt" row=1 col=2
        cell "photo.png" row=1 col=3`;

export const ORDER = `window "Order" device=phone:
  header:
    text "Your basket" bold
  list:
    item "Coffee beans, 1 kg" chevron
    item "Filter papers" chevron
  progress value=2 max=3 label="Step 2 of 3"
  button "Pay now" primary id=pay
annotation "Disabled until an address is chosen" target="pay" position=bottom`;

/**
 * `source` drawn.
 *
 * @throws Error naming the source's errors, when it isn't valid.
 */
export function drawn(source: string): HTMLElement {
  const { wireframe, errors } = parse(source);
  if (!wireframe) throw new Error(errors.map(String).join("\n"));
  return draw(wireframe, document);
}
