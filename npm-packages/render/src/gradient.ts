// A gradient in `background`, written for the text's direction (spec §9.4): its logical sides become physical ones,
// and its angle, which is logical, is mirrored in right-to-left text. The processor has checked it part by part, so
// only its first part, the direction or the shape and position, is rewritten; its stops are kept as written.

/** A text direction. */
export type Direction = "ltr" | "rtl";

const GRADIENT = /^(repeating-)?(linear|radial)-gradient\((.*)\)$/s;
const ANGLE = /^-?[\d.]+(deg|rad|grad|turn)$/;
const LOGICAL = /\b(block-start|block-end|inline-start|inline-end)\b/;

/** The physical side of each logical one, in each direction. */
const SIDES: Record<Direction, Record<string, string>> = {
  ltr: { "block-start": "top", "block-end": "bottom", "inline-start": "left", "inline-end": "right" },
  rtl: { "block-start": "top", "block-end": "bottom", "inline-start": "right", "inline-end": "left" },
};

/** `text` split at its commas outside parentheses. */
function topLevelParts(text: string): string[] {
  const parts: string[] = [];
  let depth = 0;
  let start = 0;
  for (let i = 0; i < text.length; i += 1) {
    const char = text[i];
    if (char === "(") depth += 1;
    else if (char === ")") depth -= 1;
    else if (char === "," && depth === 0) {
      parts.push(text.slice(start, i));
      start = i + 1;
    }
  }
  parts.push(text.slice(start));
  return parts.map((part) => part.trim());
}

/** A gradient's parts, and whether its first is a head (a direction, or a shape and position) rather than a stop. */
function partsOf(value: string): { name: string; head: string | undefined; stops: string[] } | undefined {
  const match = GRADIENT.exec(value);
  if (!match) return undefined;
  const [, repeating = "", kind = "", inner = ""] = match;
  const [first = "", ...rest] = topLevelParts(inner);
  const isHead = kind === "linear" ? isLinearHead(first) : isRadialHead(first);
  return { name: `${repeating}${kind}-gradient`, head: isHead ? first : undefined, stops: isHead ? rest : [first, ...rest] };
}

/** A linear gradient's first part is a head when it is an angle or starts with `to`; a stop starts with a color. */
const isLinearHead = (part: string): boolean => ANGLE.test(part) || part.split(/\s+/)[0] === "to";

/** The words that only a radial gradient's head has. */
const RADIAL_WORDS = new Set(["circle", "ellipse", "at", "closest-side", "closest-corner", "farthest-side", "farthest-corner"]);

/** A radial gradient's first part is a head when it has a shape, a size or `at`; a stop starts with a color. */
const isRadialHead = (part: string): boolean => {
  const words = part.split(/\s+/);
  return words.some((word) => RADIAL_WORDS.has(word)) || /^[\d.]+(px|rem|em|%)$/.test(words[0] ?? "");
};

/** Whether `value` is a gradient that reads differently in right-to-left text: a logical side, or an angle. */
export function followsDirection(value: string): boolean {
  const head = partsOf(value)?.head;
  return head !== undefined && (LOGICAL.test(head) || ANGLE.test(head));
}

/** A head with its logical sides made physical and, in right-to-left text, its angle mirrored. */
function physicalHead(head: string, direction: Direction): string {
  if (ANGLE.test(head)) return direction === "ltr" ? head : `calc(360deg - ${head})`;
  return head
    .split(/\s+/)
    .map((word) => SIDES[direction][word] ?? word)
    .join(" ");
}

/**
 * `value` as CSS for text in `direction`: a gradient with its logical sides made physical and its angle mirrored in
 * right-to-left text; any other value as it is.
 *
 * @example
 * gradientFor("linear-gradient(to block-end inline-end, red, blue)", "rtl");
 * // "linear-gradient(to bottom left, red, blue)"
 */
export function gradientFor(value: string, direction: Direction): string {
  const parts = partsOf(value);
  if (parts?.head === undefined) return value;
  return `${parts.name}(${[physicalHead(parts.head, direction), ...parts.stops].join(", ")})`;
}
