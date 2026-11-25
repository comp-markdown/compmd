// What the editor remembers in the browser between visits: small preferences, never the document. Storage may be
// missing or refuse (a private window, blocked site data), and then the editor only forgets.

const storage = (): Storage | undefined => {
  try {
    return window.localStorage;
  } catch {
    return undefined;
  }
};

/** What was kept under `key`, if anything. */
export function recall(key: string): string | undefined {
  try {
    return storage()?.getItem(key) ?? undefined;
  } catch {
    return undefined;
  }
}

/** Keeps `value` under `key`, when the browser lets it. */
export function keep(key: string, value: string): void {
  try {
    storage()?.setItem(key, value);
  } catch {
    // Forgotten after this visit.
  }
}
