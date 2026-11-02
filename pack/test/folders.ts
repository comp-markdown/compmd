// Packs written into temporary folders, for the tests of the command's parts.
import { mkdtemp, mkdir, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";

/** A new empty folder under the system's temporary one. */
export const folder = (): Promise<string> => mkdtemp(join(tmpdir(), "compmd-pack-"));

/** Writes `files` under `dir`, by `/`-separated path; an object is written as JSON. */
export async function write(dir: string, files: Record<string, string | object>): Promise<void> {
  for (const [path, content] of Object.entries(files)) {
    const file = join(dir, ...path.split("/"));
    await mkdir(dirname(file), { recursive: true });
    await writeFile(file, typeof content === "string" ? content : JSON.stringify(content));
  }
}

/** A pack's document importing `modules`, by their paths in the pack, and exporting `version`. */
export const documentOf = (modules: string[], version = "0.1.0"): string =>
  `---\nmd:\n  import:\n${modules.map((module) => `    - from: ./${module}\n      names: "*"`).join("\n")}\n  export:\n    pack: "@kit/charts"\n    version: "${version}"\n    names: "*"\n---\n`;
