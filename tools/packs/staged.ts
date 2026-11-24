// What npm has staged of a scope's packages (the packs', a release's), as `npm stage list --json` gives it: the
// registry's items, whose fields npm's own `npm stage list` prints (`id`, `packageName`, `version`, `status`, `tag`,
// `createdAt`, npm 11.21.1's `lib/utils/key-values.js`).

import { SCOPE } from "./placeholder.ts";

/** A version staged on npm, waiting for the author. */
export interface Staged {
  /** The stage's id, which `npm stage approve` takes. */
  id: string;
  packageName: string;
  version: string;
  /** npm's state of it; its values aren't documented. */
  status?: string;
  createdAt?: string;
}

/** A status that says the version is settled already: approved, rejected or published. */
const SETTLED = /approv|reject|publish/i;

/**
 * The staged versions of the packages of `scope` (`@compmd`) in `json`, `npm stage list --json`'s output, oldest
 * first: the order they were staged in, which is the order to approve them in, so a package is never on npm before
 * one staged before it, which it may depend on.
 *
 * @throws Error when `json` isn't a list of staged versions.
 */
export function stagedIn(json: string, scope: string): Staged[] {
  const items = JSON.parse(json) as unknown;
  if (!Array.isArray(items)) throw new Error("npm stage list --json gave no list");
  return (items as Partial<Staged>[])
    .filter((item): item is Staged => typeof item.id === "string" && typeof item.packageName === "string" && typeof item.version === "string")
    .filter((item) => item.packageName.startsWith(`${scope}/`))
    .sort((a, b) => (a.createdAt ?? "").localeCompare(b.createdAt ?? ""));
}

/** The packs' staged versions in `json`, oldest first (`stagedIn`). */
export const stagedPacks = (json: string): Staged[] => stagedIn(json, SCOPE);

/** Whether a staged version waits for the author, so the list starts with it chosen. */
export const waiting = (item: Staged): boolean => item.status === undefined || !SETTLED.test(item.status);

/** How the list shows a staged version: `@comp-markdown/card 0.1.0, staged 2026-11-20 21:04`. */
export function describe(item: Staged): string {
  const when = item.createdAt ? `, staged ${item.createdAt.slice(0, 16).replace("T", " ")}` : "";
  const status = item.status && !waiting(item) ? ` (${item.status})` : "";
  return `${item.packageName} ${item.version}${when}${status}`;
}
