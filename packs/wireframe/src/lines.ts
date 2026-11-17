// One line of a ```wireframe block, read as the grammar has it (docs/wireframe-language.md, Grammar): its indentation,
// its part's name, its arguments (strings, numbers, attributes, flags) and whether it ends in `:`.

import { WireframeError } from "./errors.ts";

/** A value, as written. */
export type Value =
  | { kind: "string"; text: string }
  | { kind: "number"; value: number; unit: "" | "px" | "%" | "fr" }
  | { kind: "range"; from: number; to: number }
  | { kind: "word"; text: string };

/** An argument of a part's line, with the column it starts at. */
export type Argument = { column: number } & ({ kind: "positional"; value: Value } | { kind: "attribute"; name: string; value: Value } | { kind: "flag"; name: string });

/** A line with a part on it. */
export interface PartLine {
  /** The line's number, from 1. */
  line: number;
  /** Its indentation, in spaces. */
  indent: number;
  name: string;
  /** The column its name starts at. */
  column: number;
  args: Argument[];
  /** Whether it ends in `:`, taking the lines indented below it as its children. */
  opens: boolean;
}

const NAME = /^[A-Za-z][A-Za-z0-9-]*/;
const WORD = /^[A-Za-z][A-Za-z0-9_-]*/;
const NUMBER = /^(\d+(?:\.\d+)?)(px|%|fr)?(?![\w.%-])/;
const RANGE = /^(\d+)-(\d+)(?![\w.%-])/;
const ESCAPES: Record<string, string> = { n: "\n", '"': '"', "\\": "\\" };

/**
 * A quoted string at `at` in `text`, with what follows it: `[text, next]`.
 *
 * @throws WireframeError for a string that doesn't end on its line, or an escape that isn't one.
 */
function stringAt(text: string, at: number, line: number): [string, number] {
  let value = "";
  for (let i = at + 1; i < text.length; i++) {
    const char = text[i] ?? "";
    if (char === '"') return [value, i + 1];
    if (char === "\\") {
      const escaped = ESCAPES[text[i + 1] ?? ""];
      if (escaped === undefined) throw new WireframeError(line, i + 1, `\`\\${text[i + 1] ?? ""}\` isn't an escape: \`\\n\`, \`\\"\` and \`\\\\\` are`);
      value += escaped;
      i++;
    } else value += char;
  }
  throw new WireframeError(line, at + 1, "a string that doesn't end: close it with `\"` on the same line");
}

/** A value at `at` in `text`, with what follows it. @throws WireframeError for what isn't one. */
function valueAt(text: string, at: number, line: number): [Value, number] {
  const rest = text.slice(at);
  if (rest.startsWith('"')) {
    const [value, next] = stringAt(text, at, line);
    return [{ kind: "string", text: value }, next];
  }
  const range = RANGE.exec(rest);
  if (range) return [{ kind: "range", from: Number(range[1]), to: Number(range[2]) }, at + range[0].length];
  const number = NUMBER.exec(rest);
  if (number) return [{ kind: "number", value: Number(number[1]), unit: (number[2] ?? "") as "" | "px" | "%" | "fr" }, at + number[0].length];
  const word = WORD.exec(rest);
  if (word) return [{ kind: "word", text: word[0] }, at + word[0].length];
  throw new WireframeError(line, at + 1, "a value is a quoted string, a number, a range (`0-100`) or a name");
}

/**
 * Line `line` of a source, `text`: `undefined` for a blank line or a comment.
 *
 * @throws WireframeError for a tab in its indentation, or what the grammar doesn't read.
 */
export function readLine(text: string, line: number): PartLine | undefined {
  const indent = /^[ \t]*/.exec(text)?.[0] ?? "";
  if (indent.includes("\t")) throw new WireframeError(line, indent.indexOf("\t") + 1, "a tab in the indentation: indent with 2 or 4 spaces");
  const body = text.slice(indent.length).trimEnd();
  if (body === "" || body.startsWith("#")) return undefined;
  const name = NAME.exec(body)?.[0];
  if (name === undefined) throw new WireframeError(line, indent.length + 1, "a line starts with a part's name, such as `row` or `button`");
  const args: Argument[] = [];
  let at = indent.length + name.length;
  let opens = false;
  while (at < text.trimEnd().length) {
    const spaces = /^ */.exec(text.slice(at))?.[0].length ?? 0;
    const start = at + spaces;
    if (text.slice(start).trimEnd() === ":") {
      opens = true;
      break;
    }
    if (spaces === 0) throw new WireframeError(line, start + 1, "arguments are separated by spaces");
    const column = start + 1;
    const rest = text.slice(start);
    const attribute = /^([A-Za-z][A-Za-z0-9-]*)=/.exec(rest);
    if (attribute?.[1]) {
      const [value, next] = valueAt(text, start + attribute[0].length, line);
      args.push({ kind: "attribute", name: attribute[1], value, column });
      at = next;
    } else if (rest.startsWith('"') || /^\d/.test(rest)) {
      const [value, next] = valueAt(text, start, line);
      args.push({ kind: "positional", value, column });
      at = next;
    } else {
      const flag = NAME.exec(rest)?.[0];
      if (flag === undefined) throw new WireframeError(line, column, `\`${rest.charAt(0)}\` can't start an argument: write a string, a number, \`name=value\` or a flag`);
      args.push({ kind: "flag", name: flag, column });
      at = start + flag.length;
    }
  }
  return { line, indent: indent.length, name, column: indent.length + 1, args, opens };
}
