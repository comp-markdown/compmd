// An installed package's manifest, found from a pack's folder as Node finds the package: in its `node_modules`, or in
// a folder above it.

import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";

/** The parts of an installed package's `package.json` the wrap reads. */
export interface Installed {
  /** Its folder. */
  dir: string;
  name: string;
  version: string;
  peerDependencies?: Record<string, string>;
  exports?: unknown;
  svelte?: string;
  module?: string;
  main?: string;
}

/**
 * The package `name` as `dir` finds it, `undefined` when it isn't installed.
 *
 * @example
 * installedPackage("qr-code", "react-qr-code")?.version; // "2.2.0"
 */
export function installedPackage(dir: string, name: string): Installed | undefined {
  for (let at = dir; ; at = dirname(at)) {
    const folder = join(at, "node_modules", ...name.split("/"));
    const manifest = join(folder, "package.json");
    if (existsSync(manifest)) return { ...(JSON.parse(readFileSync(manifest, "utf8")) as Omit<Installed, "dir">), dir: folder };
    if (dirname(at) === at) return undefined;
  }
}
