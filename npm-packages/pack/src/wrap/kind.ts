// What kind a package is, read from it (docs/wrapping-an-npm-package.md, decision 2): React with `react` among its peer
// dependencies; Svelte with `svelte` there, or a `svelte` export condition; a web component when its types define a
// custom element, or it has a `custom-elements.json`; else a plain library.

import { existsSync } from "node:fs";
import { join } from "node:path";
import type { Kind } from "./config.ts";
import type { Installed } from "./installed.ts";
import { elementTags } from "./props.ts";

/** Whether `exports` has a `svelte` condition anywhere in it. */
function hasSvelteCondition(exports: unknown): boolean {
  if (typeof exports !== "object" || exports === null) return false;
  return Object.entries(exports).some(([key, value]) => key === "svelte" || hasSvelteCondition(value));
}

/**
 * The kind of the installed package `installed`, from the pack in `dir`.
 *
 * @example
 * kindOf("qr-code", installedPackage("qr-code", "react-qr-code")); // "react"
 */
export function kindOf(dir: string, installed: Installed): Kind {
  const peers = installed.peerDependencies ?? {};
  if ("react" in peers) return "react";
  if ("svelte" in peers || installed.svelte !== undefined || hasSvelteCondition(installed.exports)) return "svelte";
  if (existsSync(join(installed.dir, "custom-elements.json")) || elementTags(dir, installed.name).length > 0) return "web-component";
  return "library";
}
