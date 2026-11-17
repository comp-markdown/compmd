// A ```wireframe block's text as an outline of parts (LANGUAGE.md, decisions 2 to 6): the lines read,
// nested by their indentation, and each part checked against its rules; every error, each with its line and column.

import { suggestion, WireframeError } from "./errors.ts";
import { readLine, type PartLine, type Value } from "./lines.ts";
import { ANNOTATION, PARTS, type PartRules, type Rule } from "./parts.ts";

/** A part of the outline. */
export interface Part {
  name: string;
  line: number;
  column: number;
  positionals: Value[];
  attributes: Record<string, Value>;
  flags: string[];
  children: Part[];
}

/** A source, parsed: its window, and the annotations after it. */
export interface Wireframe {
  window: Part;
  annotations: Part[];
}

/** What parsing gives: the wireframe when the source is valid, and its errors, in the order of the source. */
export interface Parsed {
  wireframe: Wireframe | undefined;
  errors: WireframeError[];
}

/** A value as messages write it. */
const written = (value: Value): string => {
  if (value.kind === "string") return JSON.stringify(value.text);
  if (value.kind === "number") return `${String(value.value)}${value.unit}`;
  if (value.kind === "range") return `${String(value.from)}-${String(value.to)}`;
  return value.text;
};

/** What a value must be, as messages write it. */
const wanted = (rule: Rule): string => {
  if (typeof rule !== "string") return `one of ${rule.map((name) => `\`${name}\``).join(", ")}`;
  return { text: "a string", number: "a number", integer: "a whole number from 1", range: "a range, `0-100`", value: "a value" }[rule];
};

/** Whether `value` is what `rule` takes. */
function fits(rule: Rule, value: Value): boolean {
  if (typeof rule !== "string") return (value.kind === "word" || value.kind === "string") && rule.includes(value.text);
  if (rule === "text") return value.kind === "string" || value.kind === "word";
  if (rule === "number") return value.kind === "number";
  if (rule === "integer") return value.kind === "number" && value.unit === "" && Number.isInteger(value.value) && value.value >= 1;
  if (rule === "range") return value.kind === "range";
  return true;
}

/** `line`'s part, checked against `rules`; its errors go into `errors`. */
function partOf(line: PartLine, rules: PartRules, errors: WireframeError[]): Part {
  const part: Part = { name: line.name, line: line.line, column: line.column, positionals: [], attributes: {}, flags: [], children: [] };
  const error = (column: number, problem: string) => errors.push(new WireframeError(line.line, column, problem));
  const attributes = { id: "text", ...rules.attributes } as Record<string, Rule>;
  for (const arg of line.args) {
    if (arg.kind === "positional") {
      const kind = rules.positionals?.kind;
      if (kind === undefined || part.positionals.length >= (rules.positionals?.max ?? 0)) error(arg.column, `\`${line.name}\` takes ${rules.positionals ? `at most ${String(rules.positionals.max)}` : "no"} ${kind === "width" ? "width" : "string"}${rules.positionals?.max === 1 ? "" : "s"} here: \`${written(arg.value)}\``);
      else if (kind === "string" && arg.value.kind !== "string") error(arg.column, `\`${line.name}\`'s text is a quoted string: \`"${written(arg.value)}"\``);
      else if (kind === "width" && arg.value.kind !== "number") error(arg.column, `\`${line.name}\`'s width is a number, \`240\` or \`30%\``);
      else part.positionals.push(arg.value);
    } else if (arg.kind === "attribute") {
      const rule = attributes[arg.name];
      if (rule === undefined) error(arg.column, `\`${line.name}\` has no attribute \`${arg.name}\`${suggestion(arg.name, Object.keys(attributes))}`);
      else if (arg.name in part.attributes) error(arg.column, `\`${arg.name}\` is given twice`);
      else if (!fits(rule, arg.value)) error(arg.column, `\`${arg.name}\` is ${wanted(rule)}, not \`${written(arg.value)}\``);
      else part.attributes[arg.name] = arg.value;
    } else {
      const flags = rules.flags ?? [];
      if (!flags.includes(arg.name)) {
        const known = suggestion(arg.name, [...flags, ...Object.keys(attributes)]);
        error(arg.column, `\`${line.name}\` has no flag \`${arg.name}\`${known || (flags.length === 0 ? "" : `: it has ${flags.map((flag) => `\`${flag}\``).join(", ")}`)}`);
      } else if (part.flags.includes(arg.name)) error(arg.column, `\`${arg.name}\` is given twice`);
      else part.flags.push(arg.name);
    }
  }
  const min = rules.positionals?.min ?? 0;
  if (part.positionals.length < min) error(line.column, `\`${line.name}\` needs its ${min === 2 ? "two strings" : "text, a quoted string"}`);
  for (const name of rules.required ?? []) if (!(name in part.attributes) && !line.args.some((arg) => arg.kind === "attribute" && arg.name === name)) error(line.column, `\`${line.name}\` needs \`${name}=\``);
  const range = part.attributes.range;
  if (range?.kind === "range" && range.to <= range.from) error(line.column, "`range` is from the smaller number to the larger one");
  if (line.opens && rules.children === undefined) error(line.column, `\`${line.name}\` takes no children: remove the \`:\``);
  return part;
}

/** Checks `child` may be in `parent`, as both their rules say. */
function placeChild(parent: Part, child: Part, errors: WireframeError[]): void {
  const allowed = PARTS[parent.name]?.children;
  const parents = PARTS[child.name]?.parents;
  const at = (problem: string) => errors.push(new WireframeError(child.line, child.column, problem));
  if (allowed !== undefined && allowed !== "any" && !allowed.includes(child.name)) at(`\`${parent.name}\` takes only ${allowed.map((name) => `\`${name}\``).join(" and ")}, not \`${child.name}\``);
  else if (parents !== undefined && !parents.includes(parent.name)) at(`\`${child.name}\` is only in ${parents.map((name) => `a \`${name}\``).join(" or ")}`);
}

/** The window's own rules: parts it holds at most one of, and those that exclude each other. */
function checkWindow(window: Part, errors: WireframeError[]): void {
  const named = (name: string) => window.children.filter((child) => child.name === name);
  const both = (a: string, b: string) => {
    const second = named(b)[0];
    if (named(a).length > 0 && second) errors.push(new WireframeError(second.line, second.column, `a window has a \`${a}\` or a \`${b}\`, not both`));
  };
  both("header", "navbar");
  both("footer", "tabbar");
  for (const extra of named("sheet").slice(1)) errors.push(new WireframeError(extra.line, extra.column, "a window has at most one `sheet`"));
  for (const segmented of named("segmented")) errors.push(new WireframeError(segmented.line, segmented.column, "`segmented` goes in a row or a panel, not straight in the window"));
}

/** Rules across a part's children: one `leading:` and one `trailing:` in a navbar, one selected segment. */
function checkChildren(part: Part, errors: WireframeError[]): void {
  const once = (name: string, problem: string) => {
    for (const extra of part.children.filter((child) => child.name === name).slice(1)) errors.push(new WireframeError(extra.line, extra.column, problem));
  };
  if (part.name === "navbar") {
    once("leading", "a navbar has one `leading:`: put its parts in it");
    once("trailing", "a navbar has one `trailing:`: put its parts in it");
  }
  if (part.name === "segmented") {
    for (const extra of part.children.filter((child) => child.flags.includes("selected")).slice(1)) errors.push(new WireframeError(extra.line, extra.column, "one segment is `selected`"));
  }
  for (const child of part.children) checkChildren(child, errors);
}

/** Every part's `id`, in the order of the source. */
function idsOf(part: Part, ids: Set<string>): Set<string> {
  const id = part.attributes.id;
  if (id && (id.kind === "string" || id.kind === "word")) ids.add(id.text);
  for (const child of part.children) idsOf(child, ids);
  return ids;
}

/**
 * The outline of `source`, a ```wireframe block's text, and its errors: the wireframe is there only when there are
 * none.
 *
 * @example
 * parse('window "Sign in":\n  button "Go" primary').wireframe?.window.children[0]?.flags; // ["primary"]
 */
export function parse(source: string): Parsed {
  const errors: WireframeError[] = [];
  let unit: number | undefined;
  let window: Part | undefined;
  const annotations: Part[] = [];
  /** The open parts, by depth: the last line at each, and whether it takes children. */
  const stack: { part: Part; opens: boolean }[] = [];
  for (const [i, text] of source.split(/\r?\n/).entries()) {
    let line: PartLine | undefined;
    try {
      line = readLine(text, i + 1);
    } catch (error) {
      if (!(error instanceof WireframeError)) throw error;
      errors.push(error);
      continue;
    }
    if (!line) continue;
    if (line.indent > 0 && unit === undefined) {
      if (line.indent !== 2 && line.indent !== 4) {
        errors.push(new WireframeError(line.line, 1, `the first indented line sets the indentation, 2 or 4 spaces, not ${String(line.indent)}`));
        continue;
      }
      unit = line.indent;
    }
    if (unit !== undefined && line.indent % unit !== 0) {
      errors.push(new WireframeError(line.line, 1, `indented by ${String(line.indent)} spaces: this source indents by ${String(unit)}`));
      continue;
    }
    const depth = unit === undefined ? 0 : line.indent / unit;
    const top = depth === 0;
    const rules = top && line.name === "annotation" ? ANNOTATION : PARTS[line.name];
    if (!top && line.name === "annotation") {
      errors.push(new WireframeError(line.line, line.column, "an annotation goes at the top level, after the window, not inside it"));
      continue;
    }
    if (!rules) {
      errors.push(new WireframeError(line.line, line.column, `no part \`${line.name}\`${suggestion(line.name, Object.keys(PARTS))}`));
      continue;
    }
    const part = partOf(line, rules, errors);
    stack.length = depth;
    stack.push({ part, opens: line.opens });
    if (top) {
      if (line.name === "annotation") {
        if (!window) errors.push(new WireframeError(line.line, line.column, "an annotation comes after the window"));
        annotations.push(part);
      } else if (line.name !== "window") errors.push(new WireframeError(line.line, line.column, `a source is one \`window\`; \`${line.name}\` goes indented inside it`));
      else if (window) errors.push(new WireframeError(line.line, line.column, "a source has one `window`"));
      else {
        if (annotations.length > 0) errors.push(new WireframeError(line.line, line.column, "the window comes before the annotations"));
        window = part;
      }
      continue;
    }
    const parent = stack[depth - 1];
    if (!parent) {
      errors.push(new WireframeError(line.line, 1, "indented more than one level below the line before"));
      continue;
    }
    if (!parent.opens) {
      errors.push(new WireframeError(parent.part.line, parent.part.column, `\`${parent.part.name}\` has children: end its line with \`:\``));
      parent.opens = true;
    }
    if (parent.part.name === "annotation") errors.push(new WireframeError(line.line, line.column, "an annotation takes no children"));
    placeChild(parent.part, part, errors);
    parent.part.children.push(part);
  }
  if (!window) errors.push(new WireframeError(1, 1, "a source is one `window`, with its parts indented inside it"));
  else {
    checkWindow(window, errors);
    checkChildren(window, errors);
    const ids = idsOf(window, new Set());
    for (const note of annotations) {
      const target = note.attributes.target;
      if (target && (target.kind === "string" || target.kind === "word") && !ids.has(target.text)) errors.push(new WireframeError(note.line, note.column, `no part has the id \`${target.text}\` this annotation points at`));
    }
  }
  errors.sort((a, b) => a.line - b.line || a.column - b.column);
  return { wireframe: errors.length === 0 && window ? { window, annotations } : undefined, errors };
}
