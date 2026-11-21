// A prop as a param (docs/wrapping-an-npm-package.md, decision 6, spec §7.3): its type in the spec's grammar, optional
// or with its JSDoc default, the author's overrides over it (decision 7); or why it isn't one, which the build lists.

import { BOX_PARAMS } from "@compmd/render";
import type { Override, Wrap } from "./config.ts";
import type { Prop, PropType } from "./props.ts";

/** A param of the generated signature. */
export interface Param {
  /** Its name in the signature: the prop's, or the override's. */
  name: string;
  /** The prop it is given to. */
  prop: string;
  /** Its type in the spec's grammar, without `?`. */
  type: string;
  optional: boolean;
  default?: unknown;
  doc?: string;
  /** What the build says of how it mapped: the branch of a union it took. */
  note?: string;
}

/** A prop that isn't a param, and why. */
export interface Left {
  prop: string;
  why: string;
}

/** An enum member: a name of the spec's grammar (§3.1). */
const NAME = /^[A-Za-z][A-Za-z0-9_-]*$/;

/** Props no document gives: React's and the DOM's own, and what a box or a slot gives instead. */
const NEVER: Record<string, string> = {
  ref: "React's own",
  key: "React's own",
  style: "the box's to style",
  className: "the box's to style",
  children: "content, not a value: name it in `slots` to make it one",
};

type Mapped = { type: string; note?: string } | { why: string };

function mappedType(type: PropType): Mapped {
  switch (type.kind) {
    case "string":
    case "number":
    case "boolean":
    case "date":
    case "array":
    case "object":
    case "color":
      return { type: type.kind };
    case "literals":
      return type.values.every((value) => NAME.test(value)) ? { type: type.values.join(" | ") } : { type: "string", note: "its values aren't names, so it is a string" };
    case "function":
      return { why: "a callback" };
    case "union": {
      const usable = [...new Set(type.branches.map(mappedType).flatMap((mapped) => ("type" in mapped ? [mapped.type] : [])))];
      const [only] = usable;
      if (usable.length === 1 && only !== undefined) return { type: only, note: `its ${only} branch, of ${String(type.branches.length)}` };
      return { why: usable.length === 0 ? "a union with no branch a param can be" : `a union of ${usable.join(", ")}: name its type in \`overrides\`` };
    }
    case "other":
      return { why: `its type, ${type.text}, is no param's` };
  }
}

/** A JSDoc `@default` as a value: JSON, or a quoted string, else as written. */
export function defaultValue(written: string): unknown {
  try {
    return JSON.parse(written);
  } catch {
    const quoted = /^(['"`])(.*)\1$/.exec(written);
    return quoted ? quoted[2] : written;
  }
}

/** `param` with `override` over it. */
function overridden(param: Param, override: Override | undefined): Param {
  if (!override) return param;
  return {
    ...param,
    ...(override.name === undefined ? {} : { name: override.name }),
    ...(override.type === undefined ? {} : { type: override.type }),
    ...(override.default === undefined ? {} : { default: override.default }),
    ...(override.doc === undefined ? {} : { doc: override.doc }),
  };
}

/**
 * `prop` as a param of the signature `wrap` generates, or why it isn't one.
 *
 * @example
 * mapProp({ name: "level", type: { kind: "literals", values: ["L", "M"] }, optional: true }, wrap);
 * // { name: "level", prop: "level", type: "L | M", optional: true }
 */
export function mapProp(prop: Prop, wrap: Wrap): Param | Left {
  const slot = Object.entries(wrap.slots ?? {}).find(([, given]) => given === prop.name)?.[0];
  if (slot !== undefined) return { prop: prop.name, why: `the slot \`${slot}\`` };
  const reported = Object.entries(wrap.input ?? {}).find(([, callback]) => callback === prop.name)?.[0];
  if (reported !== undefined) return { prop: prop.name, why: `reports the input \`${reported}\`` };
  const never = NEVER[prop.name];
  if (never !== undefined) return { prop: prop.name, why: never };
  if (BOX_PARAMS.has(prop.name)) return { prop: prop.name, why: "named like a common param: the use's box takes it, and the component fits its box" };
  if (prop.name === "size" && prop.type.kind === "number") return { prop: prop.name, why: "a size in pixels: the component fits its box, which the use sizes" };
  const override = wrap.overrides?.[prop.name];
  const mapped = override?.type === undefined ? mappedType(prop.type) : { type: override.type };
  if ("why" in mapped) return { prop: prop.name, why: mapped.why };
  const input = wrap.input?.[prop.name] !== undefined;
  const type = input ? `input<${mapped.type.includes("|") ? "enum" : mapped.type}>` : mapped.type;
  const param: Param = {
    name: prop.name,
    prop: prop.name,
    type,
    optional: prop.optional,
    ...(prop.default === undefined || input ? {} : { default: defaultValue(prop.default) }),
    ...(prop.doc === undefined ? {} : { doc: prop.doc }),
    ...(mapped.note === undefined ? {} : { note: mapped.note }),
  };
  return overridden(param, override);
}

/** Whether `mapped` is a param. */
export const isParam = (mapped: Param | Left): mapped is Param => "type" in mapped;
