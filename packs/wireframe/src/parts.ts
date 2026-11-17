// What each part takes (docs/wireframe-language.md, decision 4): its children, its positional arguments, its
// attributes and flags, and where it may be. Every part of Wireloom's syntax is here, the parts drawn first and the
// rest, so a source that uses any of them is checked the same way.

/** What an attribute's value is: a kind, or one of a list of names. */
export type Rule = "text" | "number" | "integer" | "range" | "value" | readonly string[];

/** A part's rules. */
export interface PartRules {
  /** Its children: any part, or those named; a leaf takes none. */
  children?: "any" | readonly string[];
  /** Its positional arguments: strings, a number (a `col`'s width), or text or number; how many are required. */
  positionals?: { kind: "string" | "width"; min: number; max: number };
  attributes?: Readonly<Record<string, Rule>>;
  /** Of those, the ones it must have. */
  required?: readonly string[];
  flags?: readonly string[];
  /** The only parts it may be in, when it isn't free to be anywhere. */
  parents?: readonly string[];
}

const ACCENTS = ["research", "military", "industry", "wealth", "approval", "warning", "danger", "success"] as const;
const STATES = ["locked", "available", "active", "purchased", "maxed", "growing", "ripe", "withering", "cashed"] as const;
const TYPOGRAPHY = { weight: ["light", "regular", "semibold", "bold"], size: ["small", "regular", "large"] } as const;
const STYLE_FLAGS = ["bold", "italic", "muted"] as const;
const one = (min: 0 | 1 = 1) => ({ kind: "string", min, max: 1 }) as const;

/** The parts, by name. */
export const PARTS: Readonly<Record<string, PartRules>> = {
  // Containers.
  window: { children: "any", positionals: one(0), attributes: { device: ["phone", "tablet", "browser", "desktop"] } },
  header: { children: "any", flags: ["large"] },
  footer: { children: "any" },
  navbar: { children: ["leading", "trailing"], parents: ["window"] },
  leading: { children: "any", parents: ["navbar"] },
  trailing: { children: "any", parents: ["navbar"] },
  tabbar: { children: ["tabitem"], parents: ["window"] },
  sheet: { children: "any", attributes: { position: ["bottom", "center"], title: "text" }, parents: ["window"] },
  panel: { children: "any" },
  section: { children: "any", positionals: one(), attributes: { badge: "text", accent: ACCENTS } },
  tabs: { children: ["tab"] },
  row: { children: "any", attributes: { align: ["left", "center", "right"], justify: ["start", "between", "around", "end"] } },
  col: { children: "any", positionals: { kind: "width", min: 0, max: 1 }, flags: ["fill"] },
  list: { children: ["item", "slot"] },
  slot: { children: "any", positionals: one(), attributes: { state: STATES, accent: ACCENTS }, flags: ["active", "chevron"] },
  grid: { children: ["cell"], attributes: { cols: "integer", rows: "integer" }, required: ["cols", "rows"] },
  resourcebar: { children: ["resource"] },
  stats: { children: ["stat"] },
  segmented: { children: ["segment"] },
  tree: { children: ["node"] },
  menubar: { children: ["menu"] },
  menu: { children: ["menuitem", "separator"], positionals: one() },
  breadcrumb: { children: ["crumb"] },
  // Leaves.
  tab: { positionals: one(), attributes: { badge: "text" }, flags: ["active"], parents: ["tabs"] },
  item: { positionals: one(), flags: ["chevron"], parents: ["list"] },
  text: { positionals: one(), attributes: { ...TYPOGRAPHY, accent: ACCENTS }, flags: STYLE_FLAGS },
  button: { positionals: one(0), attributes: { badge: "text", accent: ACCENTS, icon: "text" }, flags: ["primary", "disabled"] },
  backbutton: { positionals: one(0), flags: ["disabled"] },
  input: { attributes: { placeholder: "text", type: "text" }, flags: ["disabled"] },
  combo: { positionals: one(0), attributes: { value: "text", options: "text" }, flags: ["disabled"] },
  slider: { attributes: { range: "range", value: "number", label: "text" }, required: ["range", "value"] },
  kv: { positionals: { kind: "string", min: 2, max: 2 }, attributes: { ...TYPOGRAPHY, icon: "text", accent: ACCENTS }, flags: STYLE_FLAGS },
  image: { attributes: { label: "text", width: "number", height: "number" } },
  icon: { attributes: { name: "text", accent: ACCENTS } },
  divider: {},
  cell: { children: "any", positionals: one(0), attributes: { row: "integer", col: "integer", state: STATES, accent: ACCENTS }, required: ["row", "col"], parents: ["grid"] },
  resource: { attributes: { name: "text", value: "value", icon: "text" }, required: ["name", "value"], parents: ["resourcebar"] },
  stat: { positionals: { kind: "string", min: 2, max: 2 }, attributes: { icon: "text", accent: ACCENTS }, flags: ["bold", "muted"], parents: ["stats"] },
  progress: { attributes: { value: "number", max: "number", label: "text", accent: ACCENTS } },
  chart: { attributes: { kind: ["bar", "line", "pie"], label: "text", width: "number", height: "number", accent: ACCENTS } },
  spacer: { parents: ["row"] },
  tabitem: { positionals: one(), attributes: { icon: "text", badge: "text" }, flags: ["selected", "disabled"], parents: ["tabbar"] },
  segment: { positionals: one(), flags: ["selected", "disabled"], parents: ["segmented"] },
  checkbox: { positionals: one(), flags: ["checked", "disabled", "label-right"] },
  radio: { positionals: one(), attributes: { group: "text" }, flags: ["selected", "disabled", "label-right"] },
  toggle: { positionals: one(), flags: ["on", "off", "disabled", "label-right"] },
  menuitem: { positionals: one(), attributes: { shortcut: "text" }, flags: ["disabled"], parents: ["menu"] },
  separator: {},
  node: { children: ["node"], positionals: one(), attributes: { icon: "text" }, flags: ["collapsed", "selected"], parents: ["tree", "node"] },
  crumb: { positionals: one(), attributes: { icon: "text" }, parents: ["breadcrumb"] },
  chip: { positionals: one(), attributes: { accent: ACCENTS, icon: "text" }, flags: ["closable", "selected"] },
  avatar: { positionals: one(), attributes: { size: ["small", "medium", "large"], accent: ACCENTS } },
  spinner: { positionals: one(0) },
  status: { positionals: one(), attributes: { kind: ["success", "info", "warning", "error"] }, required: ["kind"] },
};

/** The top-level line after the window: a note pointing at a part (decision 5). */
export const ANNOTATION: PartRules = { positionals: one(), attributes: { target: "text", position: ["left", "right", "top", "bottom"] }, required: ["target", "position"] };

/** The parts the pack draws first (decision 4); the others are drawn as a named box until it draws them. */
export const DRAWN = new Set([
  "window", "header", "footer", "panel", "section", "row", "col", "grid", "cell", "tabs", "tab", "list", "item", "tree", "node",
  "menubar", "menu", "menuitem", "separator", "divider", "text", "button", "input", "combo", "checkbox", "radio", "toggle",
  "progress", "image", "icon", "spacer",
]);
