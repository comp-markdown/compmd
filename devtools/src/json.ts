// What the JSON tree shows of a value: whether it folds, its entries, and a summary of it while folded.

/** An object or an array: what folds. */
export const isBranch = (value: unknown): value is object => typeof value === "object" && value !== null;

/** A branch's entries, by key or index. */
export function entriesOf(value: object): [string, unknown][] {
  return Array.isArray(value) ? value.map((item, index): [string, unknown] => [String(index), item]) : Object.entries(value);
}

/** The fields that tell a tree node apart, after its type. */
const TELLING = ["name", "depth", "lang", "identifier", "key"];

/** How many of a record's first values its summary shows. */
const SHOWN = 3;

/**
 * A folded branch, in a few words: an array's length; a tree node's type with its telling fields; a record's first
 * plain values, so a list of records reads at a glance; otherwise its number of fields.
 *
 * @example
 * summaryOf({ type: "heading", depth: 2, children: [] }); // "heading (depth 2)"
 * summaryOf([1, 2, 3]); // "[3]"
 * summaryOf({ month: "2026-01", units: 4 }); // '{month: "2026-01", units: 4}'
 */
export function summaryOf(value: object): string {
  if (Array.isArray(value)) return `[${String(value.length)}]`;
  const record = value as Record<string, unknown>;
  if (typeof record.type === "string") {
    const telling = TELLING.filter((field) => typeof record[field] === "string" || typeof record[field] === "number").map((field) => `${field} ${String(record[field])}`);
    return telling.length > 0 ? `${record.type} (${telling.join(", ")})` : record.type;
  }
  const entries = Object.entries(record);
  const plain = entries.filter(([, field]) => !isBranch(field));
  if (plain.length === 0) return `{${String(entries.length)} ${entries.length === 1 ? "field" : "fields"}}`;
  const shown = plain.slice(0, SHOWN).map(([key, field]) => `${key}: ${leafText(field)}`);
  return `{${shown.join(", ")}${entries.length > shown.length ? ", …" : ""}}`;
}

/** A leaf as JSON writes it: strings quoted. */
export const leafText = (value: unknown): string => (value === undefined ? "undefined" : JSON.stringify(value));
