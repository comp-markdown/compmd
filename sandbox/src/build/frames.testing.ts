// What the build's tests share: new folders, removed after each test, and a stand-in for the package's built frame
// files, which `buildFrames` reads (`from`).

import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach } from "vitest";

const folders: string[] = [];
afterEach(async () => {
  for (const folder of folders.splice(0)) await rm(folder, { recursive: true, force: true });
});

/** A new empty folder, removed after the test. */
export async function folder(): Promise<string> {
  const made = await mkdtemp(join(tmpdir(), "compmd-sandbox-build-"));
  folders.push(made);
  return made;
}

/** A stand-in for the package's built frame files: the runtime, and React as the one shared library. */
export async function builtFrames(): Promise<string> {
  const from = await folder();
  await mkdir(join(from, "libs"), { recursive: true });
  await writeFile(join(from, "runtime.js"), "// the frame runtime\n");
  await writeFile(join(from, "libs/importmap.json"), JSON.stringify({ imports: { react: "./libs/react@19.3.0/index.js" } }));
  await writeFile(join(from, "libs/libraries.json"), JSON.stringify([{ name: "react", version: "19.3.0", imports: ["react"] }]));
  return from;
}
