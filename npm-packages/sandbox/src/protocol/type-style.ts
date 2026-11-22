// The `type-style` message's style and fonts, as a frame takes them: each field of its type, or nothing, since the
// message comes from the page and a frame trusts no message's shape.
import type { TypeStyle } from "@compmd/render";
import type { FrameFont } from "./messages.ts";

/** The face names `FontFaces` gives a document's own fonts (`compmd-font-<n>-<family>`), and nothing a rule could read as more. */
const FACE = /^compmd-font-\d+-[A-Za-z0-9_-]+$/;

/** `value` as the files of a box's fonts: each one that is one, a face's name, a weight, a style and a Blob. */
export function frameFontsIn(value: unknown): FrameFont[] {
  if (!Array.isArray(value)) return [];
  return value.filter((font: unknown): font is FrameFont => {
    if (typeof font !== "object" || font === null) return false;
    const { name, weight, style, file } = font as Record<string, unknown>;
    return typeof name === "string" && FACE.test(name) && (weight === "normal" || weight === "bold") && (style === "normal" || style === "italic") && file instanceof Blob;
  });
}

const TEXTS = ["family", "weight", "style", "color", "lineHeight", "letterSpacing"] as const;

/** `value` as a box's type, when it is one: its texts strings and its size a positive number. */
export function typeStyleIn(value: unknown): TypeStyle | undefined {
  if (typeof value !== "object" || value === null) return undefined;
  const given = value as Record<string, unknown>;
  if (typeof given.size !== "number" || !(given.size > 0) || !TEXTS.every((key) => typeof given[key] === "string")) return undefined;
  const text = (key: (typeof TEXTS)[number]) => given[key] as string;
  return { family: text("family"), size: given.size, weight: text("weight"), style: text("style"), color: text("color"), lineHeight: text("lineHeight"), letterSpacing: text("letterSpacing") };
}
