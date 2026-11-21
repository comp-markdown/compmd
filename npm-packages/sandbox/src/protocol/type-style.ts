// The `type-style` message's style, as a frame takes it: each field of its type, or nothing, since the message comes
// from the page and a frame trusts no message's shape.
import type { TypeStyle } from "@compmd/render";

const TEXTS = ["family", "weight", "style", "color", "lineHeight", "letterSpacing"] as const;

/** `value` as a box's type, when it is one: its texts strings and its size a positive number. */
export function typeStyleIn(value: unknown): TypeStyle | undefined {
  if (typeof value !== "object" || value === null) return undefined;
  const given = value as Record<string, unknown>;
  if (typeof given.size !== "number" || !(given.size > 0) || !TEXTS.every((key) => typeof given[key] === "string")) return undefined;
  const text = (key: (typeof TEXTS)[number]) => given[key] as string;
  return { family: text("family"), size: given.size, weight: text("weight"), style: text("style"), color: text("color"), lineHeight: text("lineHeight"), letterSpacing: text("letterSpacing") };
}
