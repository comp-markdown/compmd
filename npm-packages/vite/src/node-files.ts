// Node's file system as `compmd-project` reads it (`@compmd/host/project`, which loads its module itself in Node).
// Node only.

import { readFileSync, statSync } from "node:fs";
import type { ProjectFiles } from "@compmd/host/project";

/** The disk's files: a file's text, or `undefined` for none or a folder; whether a path is a folder. */
export const nodeFiles: ProjectFiles = {
  text(path) {
    try {
      return readFileSync(path, "utf8");
    } catch {
      return undefined;
    }
  },
  isDir: (path) => statSync(path, { throwIfNoEntry: false })?.isDirectory() ?? false,
};
