// The hint line above the diagnostics: what the processor's positional hint at the caret says, in a few short parts,
// with the values it offers as buttons.

import type { Completion, ComponentHint, Declaration, ParameterHint, PositionalHint } from "@compmd/processor";
import { paramText } from "./components.ts";

/** A run of the line's text: plain, code, or the name the hint is about. */
export interface Run {
  text: string;
  code?: true;
  name?: true;
}

/** What the hint line shows: its parts, which it separates with ` · `, and the values it offers, the written one marked. */
export interface HintLine {
  parts: Run[][];
  values: Completion[];
  current?: string;
}

/** `text` with its backtick spans as code, as the processor's messages write them. */
export function runs(text: string): Run[] {
  return text
    .split("`")
    .map((part, i): Run => (i % 2 === 1 ? { text: part, code: true } : { text: part }))
    .filter((run) => run.text !== "");
}

const named = (text: string): Run => ({ text, name: true });
const code = (text: string): Run => ({ text, code: true });

/** `items` as code, joined by commas. */
function listed(items: readonly string[]): Run[] {
  return items.flatMap((item, i) => (i === 0 ? [code(item)] : [{ text: ", " }, code(item)]));
}

/** A declaration as its short form writes it, with its default: `kind: note | warning = note`. */
function declarationText(declaration: Declaration): string {
  const value = declaration.default;
  const written = value === null ? "" : ` = ${typeof value.value === "string" ? value.value : JSON.stringify(value.value)}`;
  return `${paramText(declaration)}${written}`;
}

/** What the component is and where it comes from: "a CompMD component defined in this document (line 3)". */
function origin(component: ComponentHint): string {
  const what = component.kind === "compmd" ? "a CompMD component" : component.kind === "processor" ? "a built-in" : `a \`${component.kind}\` component`;
  if (component.file === undefined) return `${what} defined in this document${component.defined ? ` (line ${String(component.defined.start.line)})` : ""}`;
  // A pack's file is `@scope/name@version/path`: the pack is what a reader knows.
  const pack = /^(@[^/]+\/[^@/]+)@/.exec(component.file)?.[1];
  return `${what} from \`${pack ?? component.file}\``;
}

function componentParts(name: string, component: ComponentHint | undefined): Run[][] {
  if (!component) return [[named(name), { text: " stands for no component here" }]];
  const parts: Run[][] = [[named(name), { text: ", " }, ...runs(origin(component))]];
  const signature = component.signature;
  if (!signature) return [...parts, [{ text: "no signature: its params go unchecked" }]];
  if (signature.params.length > 0) parts.push([{ text: "params " }, ...listed(signature.params.map(declarationText))]);
  parts.push(signature.slots ? [{ text: "slots " }, ...listed(signature.slots.map((slot) => (typeof slot === "string" ? slot : slot.name)))] : [{ text: "no slots" }]);
  return parts;
}

/** What a param is, on its key: what it takes, its default, its `doc`, whether it takes a breakpoint. */
function paramParts(component: string, param: ParameterHint, onValue: boolean): Run[][] {
  const kind = param.origin === "common" ? ", a common param" : param.origin === "attribute" ? ", an attribute" : "";
  const parts: Run[][] = [[named(param.name), { text: " on " }, code(component), { text: kind }]];
  if (onValue && param.takes.tokens.length > 0 && param.takes.forms.length === 0) return parts;
  parts.push(runs(`takes ${param.takes.description}`));
  if (onValue) return parts;
  if (param.default !== undefined) parts.push([{ text: "default " }, code(param.default)]);
  if (param.required) parts.push([{ text: "required" }]);
  if (param.doc) parts.push([{ text: param.doc }]);
  if (param.breakpoints) parts.push([{ text: "takes a breakpoint, " }, code(`md:${param.name}=…`)]);
  return parts;
}

/** The hint line for `hint`, or `undefined` when it has no subject: its diagnostics alone say it. */
export function lineOf(hint: PositionalHint): HintLine | undefined {
  const subject = hint.subject;
  if (!subject) return undefined;
  const line = (parts: Run[][], values: Completion[] = [], current?: string): HintLine => ({ parts, values, ...(current === undefined ? {} : { current }) });
  switch (subject.kind) {
    case "component":
      return line(componentParts(subject.name, subject.component));
    case "params":
      return line([[code(subject.component), { text: " takes " }, ...listed(subject.params.map((param) => param.name))]]);
    case "param":
      return subject.value === undefined
        ? line(paramParts(subject.component, subject.param, false))
        : line(paramParts(subject.component, subject.param, true), hint.completions, subject.value);
    case "slot": {
      const slots = subject.slots.map((slot) => (typeof slot === "string" ? slot : slot.name));
      const declared = slots.length > 0 ? [code(subject.component), { text: " has slots " }, ...listed(slots)] : [code(subject.component), { text: " declares no slots" }];
      return line([[{ text: "slot " }, named(subject.slot), { text: " of " }, code(subject.component)], declared]);
    }
    case "path": {
      const { name } = subject;
      const parts: Run[][] = [[named(subject.path)]];
      if (name.about) parts.push(runs(name.about));
      parts.push(name.type === undefined ? [{ text: "nothing here has this name" }] : [code(name.type)]);
      if (name.fields) parts.push([{ text: "fields " }, ...listed(name.fields)]);
      return line(parts);
    }
    case "function":
      return subject.hint
        ? line([[named(subject.name)], listed(subject.hint.signatures), runs(subject.hint.doc)])
        : line([[named(subject.name), { text: " is no function of the profile" }]]);
    case "frontmatter":
      return line(subject.about ? [[named(subject.path.join("."))], runs(subject.about)] : [[named(subject.path.join("."))]]);
    case "type":
      return line([[{ text: "type " }, named(subject.name)], subject.takes ? runs(subject.takes.description) : [{ text: "no such type" }]], hint.completions, subject.name);
  }
}
