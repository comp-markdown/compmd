// The parts drawn as the controls they stand for (LANGUAGE.md, decision 5), in greys.

import { attribute, badge, childrenOf, element, flag, flagged, icon, numberOf, textOf, type DrawPart } from "./dom.ts";

/** `text`: a line of text, its style from its flags, `weight` and `size`. */
export const text: DrawPart = (part, drawing) => {
  const weight = attribute(part, "weight");
  const size = attribute(part, "size");
  const names = ["text", ...flagged(part, ["bold", "italic", "muted"]), ...(weight ? [`weight-${weight}`] : []), ...(size ? [`size-${size}`] : [])];
  return element(drawing.document, "div", names, textOf(part) ?? "");
};

/** `button`: a bordered box, filled when `primary`. */
export const button: DrawPart = (part, drawing) =>
  element(drawing.document, "span", ["button", ...flagged(part, ["primary", "disabled"])], icon(drawing.document, attribute(part, "icon")), textOf(part), badge(drawing.document, attribute(part, "badge")));

/** `input`: a box with its placeholder muted, or dots for a password. */
export const input: DrawPart = (part, drawing) => {
  const password = attribute(part, "type") === "password";
  const shown = password ? element(drawing.document, "span", ["dots-text"], "••••••••") : element(drawing.document, "span", ["placeholder"], attribute(part, "placeholder") ?? "");
  return element(drawing.document, "span", ["input", ...flagged(part, ["disabled"])], shown);
};

/** A field's label, when it has one. */
const label = (document: Document, text: string | undefined) => (text === undefined ? undefined : element(document, "span", ["label"], text));

/** `combo`: its label, then a box with its value (or its first option) and a chevron. */
export const combo: DrawPart = (part, drawing) => {
  const value = attribute(part, "value") ?? attribute(part, "options")?.split(",")[0]?.trim() ?? "";
  const box = element(drawing.document, "span", ["combo-box"], element(drawing.document, "span", ["value"], value), element(drawing.document, "span", ["chevron"], "▾"));
  return element(drawing.document, "span", ["combo", ...flagged(part, ["disabled"])], label(drawing.document, textOf(part)), box);
};

/** A labelled mark: its label, then its mark, or the other way round with `label-right`. */
const marked =
  (name: string, on: string): DrawPart =>
  (part, drawing) => {
    const mark = element(drawing.document, "span", ["mark", `${name}-mark`, ...(flag(part, on) ? ["on"] : [])]);
    const text = label(drawing.document, textOf(part));
    const parts = flag(part, "label-right") ? [mark, text] : [text, mark];
    return element(drawing.document, "span", ["choice", name, ...flagged(part, ["disabled"])], ...parts);
  };

export const checkbox = marked("checkbox", "checked");
export const radio = marked("radio", "selected");
export const toggle = marked("toggle", "on");

/** `progress`: a bar filled to `value` of `max`, its label above it. */
export const progress: DrawPart = (part, drawing) => {
  const max = numberOf(part, "max") ?? 100;
  const value = numberOf(part, "value") ?? 0;
  const fill = element(drawing.document, "span", ["progress-fill"]);
  fill.style.width = `${String(max > 0 ? Math.min(100, Math.max(0, (value / max) * 100)) : 0)}%`;
  return element(drawing.document, "div", ["progress"], label(drawing.document, attribute(part, "label")), element(drawing.document, "span", ["progress-bar"], fill));
};

/** `tabs`: a row of tabs over a line. */
export const tabs: DrawPart = (part, drawing) => element(drawing.document, "div", ["tabs"], ...childrenOf(part, drawing));

export const tab: DrawPart = (part, drawing) => element(drawing.document, "span", ["tab", ...flagged(part, ["active"])], textOf(part) ?? "", badge(drawing.document, attribute(part, "badge")));

/** `list`: its items, one under another. */
export const list: DrawPart = (part, drawing) => element(drawing.document, "ul", ["list"], ...childrenOf(part, drawing));

export const item: DrawPart = (part, drawing) =>
  element(drawing.document, "li", ["item"], element(drawing.document, "span", ["item-text"], textOf(part) ?? ""), flag(part, "chevron") ? element(drawing.document, "span", ["chevron"], "›") : undefined);

/** `tree`: an indented list with chevrons. */
export const tree: DrawPart = (part, drawing) => element(drawing.document, "ul", ["tree"], ...childrenOf(part, drawing));

/** `node`: a chevron when it has children (closed when `collapsed`), its icon and label, then its children unless collapsed. */
export const node: DrawPart = (part, drawing) => {
  const collapsed = flag(part, "collapsed");
  const open = part.children.length > 0 && !collapsed;
  const chevron = element(drawing.document, "span", ["chevron"], part.children.length > 0 || collapsed ? (collapsed ? "▸" : "▾") : "");
  const row = element(drawing.document, "span", ["node-label", ...flagged(part, ["selected"])], chevron, icon(drawing.document, attribute(part, "icon")), textOf(part) ?? "");
  return element(drawing.document, "li", ["node"], row, open ? element(drawing.document, "ul", ["tree"], ...childrenOf(part, drawing)) : undefined);
};

/** `menubar`: a bar of its menus' titles. */
export const menubar: DrawPart = (part, drawing) => element(drawing.document, "div", ["menubar"], ...childrenOf(part, drawing));

/** `menu`: its title, and its items drawn open below it. */
export const menu: DrawPart = (part, drawing) =>
  element(
    drawing.document,
    "span",
    ["menu"],
    element(drawing.document, "span", ["menu-title"], textOf(part) ?? ""),
    part.children.length > 0 ? element(drawing.document, "span", ["menu-items"], ...childrenOf(part, drawing)) : undefined,
  );

export const menuitem: DrawPart = (part, drawing) => {
  const shortcut = attribute(part, "shortcut");
  return element(
    drawing.document,
    "span",
    ["menuitem", ...flagged(part, ["disabled"])],
    element(drawing.document, "span", ["menuitem-label"], textOf(part) ?? ""),
    shortcut === undefined ? undefined : element(drawing.document, "span", ["shortcut"], shortcut),
  );
};

/** `separator` and `divider`: a line. */
export const divider: DrawPart = (_part, drawing) => {
  const line = element(drawing.document, "span", ["divider"]);
  line.setAttribute("role", "separator");
  return line;
};

/** `image`: a crossed box of its size. */
export const image: DrawPart = (part, drawing) => {
  const drawn = element(drawing.document, "span", ["image"], element(drawing.document, "span", ["image-label"], attribute(part, "label") ?? ""));
  const width = numberOf(part, "width");
  const height = numberOf(part, "height");
  if (width !== undefined) drawn.style.width = `${String(width)}px`;
  drawn.style.height = `${String(height ?? 96)}px`;
  return drawn;
};

/** `icon`: a small square with its name's first letter. */
export const iconPart: DrawPart = (part, drawing) => icon(drawing.document, attribute(part, "name") ?? "?") ?? element(drawing.document, "span", ["icon"]);

/** `spacer`: the row's slack. */
export const spacer: DrawPart = (_part, drawing) => element(drawing.document, "span", ["spacer"]);
