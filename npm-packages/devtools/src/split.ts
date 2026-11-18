// The split between the text and the views: where it is, as the text's share of the width, and which side is folded
// away, if one is. The browser remembers it.

/** Where the split is: the text's share of the width, and the side folded away, if any. */
export interface Split {
  at: number;
  folded: "text" | "views" | undefined;
}

/** The narrowest share a side keeps while it is open. */
export const MIN = 0.2;
export const MAX = 0.8;
const STEP = 0.02;
const KEY = "compmd-editor-split";

export const DEFAULT: Split = { at: 0.42, folded: undefined };

/** `at` within the shares a side may have. */
export const clamp = (at: number): number => Math.min(MAX, Math.max(MIN, at));

/** The split after a key on the handle, or `undefined` for a key it doesn't take. */
export function keyed(split: Split, key: string): Split | undefined {
  switch (key) {
    case "ArrowLeft":
      return { at: clamp(split.at - STEP), folded: undefined };
    case "ArrowRight":
      return { at: clamp(split.at + STEP), folded: undefined };
    case "Home":
      return { ...split, folded: "text" };
    case "End":
      return { ...split, folded: "views" };
    case "Enter":
      return { ...split, folded: undefined };
    default:
      return undefined;
  }
}

/** The split `storage` remembers, or the default; storage may be missing or refuse. */
export function load(storage: Pick<Storage, "getItem"> | undefined): Split {
  try {
    const saved = JSON.parse(storage?.getItem(KEY) ?? "null") as Partial<Split> | null;
    const folded = saved?.folded === "text" || saved?.folded === "views" ? saved.folded : undefined;
    return typeof saved?.at === "number" ? { at: clamp(saved.at), folded } : DEFAULT;
  } catch {
    return DEFAULT;
  }
}

/** Remembers `split` in `storage`, when it may. */
export function save(storage: Pick<Storage, "setItem"> | undefined, split: Split): void {
  try {
    storage?.setItem(KEY, JSON.stringify(split));
  } catch {
    // A private window or blocked storage: the split is only for this visit.
  }
}
