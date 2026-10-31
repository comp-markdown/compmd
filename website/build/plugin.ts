// The Vite plugin that gives the page its snapshots, as `import snapshots, { files } from "virtual:snapshots"`, made
// from the documents in `content/` when the page is built, and again when one of them changes. `files` are the other
// files there, which the documents read, for the live parts.
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { loadApp } from "@compmd/vite";
import type { Plugin } from "vite";
import { EXTENSION, snapshots } from "./snapshots.ts";

const ID = "virtual:snapshots";
const RESOLVED = `\0${ID}`;

/**
 * Serves `virtual:snapshots` from the documents in `folder`, rendered with the packs the site at `root` gives them
 * (its `compmd.config.json`), as the live page renders them.
 */
export function snapshotsOf(folder: string, root: string): Plugin {
  return {
    name: "compmd-snapshots",
    resolveId: (id) => (id === ID ? RESOLVED : undefined),
    async load(id) {
      if (id !== RESOLVED) return undefined;
      const names = readdirSync(folder);
      for (const name of names) this.addWatchFile(join(folder, name));
      const files = Object.fromEntries(names.filter((name) => !name.endsWith(EXTENSION)).map((name) => [name, readFileSync(join(folder, name), "utf8")]));
      const { request } = await loadApp(root);
      return `export default ${JSON.stringify(snapshots(folder, new Date().toISOString(), request))};\nexport const files = ${JSON.stringify(files)};`;
    },
  };
}
