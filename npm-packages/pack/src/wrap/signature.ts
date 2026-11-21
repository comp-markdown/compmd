// The generated signature fence (docs/wrapping-an-npm-package.md, decision 4; spec §7.3): the params in the short form,
// `level: L | M | H | Q?` or `margin: number = 4`, and in the long form when one has a doc; the slots the overrides
// name; and the box's defaults as the fence's param list (G8).

import { stringify } from "yaml";
import type { Wrap } from "./config.ts";
import type { Param } from "./map.ts";

/** A default as a param value (spec §3.1): bare when it can be, else quoted or JSON. */
export function writtenValue(value: unknown): string {
  if (typeof value === "string") return /^[^\s"'{}()[\]=,]+$/.test(value) ? value : JSON.stringify(value);
  if (typeof value === "number" || typeof value === "boolean") return String(value);
  return JSON.stringify(value);
}

/** `param` as its declaration: the short form's text, or the long form's mapping. */
function declaration(param: Param): string | Record<string, unknown> {
  const type = `${param.type}${param.optional && param.default === undefined ? "?" : ""}`;
  if (param.doc === undefined) return param.default === undefined ? type : `${type} = ${writtenValue(param.default)}`;
  return { type, ...(param.default === undefined ? {} : { default: param.default }), doc: param.doc };
}

/**
 * The signature fence of the component `component` for `params`.
 *
 * @example
 * signatureFence("qr-code", [{ name: "value", prop: "value", type: "string", optional: false }], wrap);
 * // "```compmd component=qr-code signature\nparams:\n  value: string\n```"
 */
export function signatureFence(component: string, params: readonly Param[], wrap: Wrap): string {
  const header = {
    params: Object.fromEntries(params.map((param) => [param.name, declaration(param)])),
    ...(wrap.slots === undefined ? {} : { slots: Object.keys(wrap.slots) }),
  };
  const yaml = stringify(header, { lineWidth: 0, flowCollectionPadding: false }).trimEnd();
  const box = wrap.box === undefined || wrap.box.trim() === "" ? "" : ` {${wrap.box.trim()}}`;
  return `\`\`\`compmd component=${component} signature${box}\n${yaml}\n\`\`\``;
}
