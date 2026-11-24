// A wrapped package's settings, `compmd.wrap.json` beside `package.json` (docs/wrapping-an-npm-package.md, decisions 5
// and 7): the package and its export, its kind, the allowlist of the props that are params, and the overrides. The pack
// doesn't publish it: `package.json`'s `compmd` is the pack's document.

import { existsSync, readFileSync } from "node:fs";
import { messageOf } from "../message-of.ts";
import { join } from "node:path";
import { PackError } from "../pack.ts";

/** The file's name, in the pack's folder. */
export const WRAP_FILE = "compmd.wrap.json";

/** What a package is, which decides its glue (decision 2). */
export type Kind = "react" | "svelte" | "web-component" | "library";

export const KINDS: readonly Kind[] = ["react", "svelte", "web-component", "library"];

/** What the author says of one param, over what the types give (decision 7). */
export interface Override {
  /** The param's name, when it isn't the prop's (`fgColor` as `color`). */
  name?: string;
  /** Its type, in the spec's grammar (§7.3), without `?`. */
  type?: string;
  /** Its default, any YAML value of its type. */
  default?: unknown;
  /** Its description, for editors. */
  doc?: string;
}

/** A wrapped package's settings. */
export interface Wrap {
  /** The npm package. */
  from: string;
  /** The export that is the component: `default`, or a name (`BarChart`); a web component's element, by its tag. */
  export: string;
  kind: Kind;
  /** The props that are params, by the prop's name: a prop the package adds later is reported, never added (decision 5). */
  params: string[];
  overrides?: Record<string, Override>;
  /** The props that are inputs, each with the callback that reports its new value: `{ "value": "onChange" }`. */
  input?: Record<string, string>;
  /** The prop that takes the fallback's text when the document gives it none (a `title`, an `alt`). */
  label?: string;
  /** The props that are slots, by the slot's name: `{ "default": "children" }`. */
  slots?: Record<string, string>;
  /** The generated signature fence's param list, the box's defaults (`width=12rem max-width=100%`). */
  box?: string;
}

const isStrings = (value: unknown): value is string[] => Array.isArray(value) && value.every((item) => typeof item === "string");
const isRecord = (value: unknown): value is Record<string, unknown> => typeof value === "object" && value !== null && !Array.isArray(value);
const isStringRecord = (value: unknown): value is Record<string, string> => isRecord(value) && Object.values(value).every((item) => typeof item === "string");

/** Why `value` isn't a wrap's settings, or `undefined` when it is. */
export function problemOfWrap(value: unknown): string | undefined {
  if (!isRecord(value)) return "it isn't a JSON object";
  if (typeof value.from !== "string") return "`from` names the npm package it wraps";
  if (typeof value.export !== "string") return '`export` names the component: "default", or the export\'s name';
  if (!KINDS.includes(value.kind as Kind)) return `\`kind\` is one of ${KINDS.join(", ")}`;
  if (!isStrings(value.params)) return "`params` lists the props that are params, by name";
  if (value.overrides !== undefined && !(isRecord(value.overrides) && Object.values(value.overrides).every(isRecord))) return "`overrides` holds an object for each param it changes";
  for (const key of ["input", "slots"] as const) if (value[key] !== undefined && !isStringRecord(value[key])) return `\`${key}\` maps names to names`;
  for (const key of ["label", "box"] as const) if (value[key] !== undefined && typeof value[key] !== "string") return `\`${key}\` is a string`;
  return undefined;
}

/**
 * The settings of the pack in `dir`, `undefined` when it wraps no package.
 *
 * @throws PackError when the file isn't JSON, or isn't a wrap's settings.
 */
export function readWrap(dir: string): Wrap | undefined {
  const file = join(dir, WRAP_FILE);
  if (!existsSync(file)) return undefined;
  let value: unknown;
  try {
    value = JSON.parse(readFileSync(file, "utf8"));
  } catch (error) {
    throw new PackError(`${WRAP_FILE} isn't JSON: ${messageOf(error)}`);
  }
  const problem = problemOfWrap(value);
  if (problem) throw new PackError(`${WRAP_FILE}: ${problem}`);
  return value as Wrap;
}
