// What a blog's posts with host-language components need (docs/blog-tool.md, decision 5): the blog's packs, as
// `@compmd/vite` reads them from its `compmd.config.json`, for processing; and the page's script with the sandbox's
// frames, built by Vite with `@compmd/vite`'s plugin, for the pages.

import { existsSync } from "node:fs";
import { mkdtemp, readFile, readdir, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { extname, join, relative, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { compmd, loadApp } from "@compmd/vite";
import { build } from "vite";
import { BlogError } from "./config.ts";
import type { Given } from "./render.ts";

/** The packs' configuration in a blog's folder, as any app's. */
export const PACKS = "compmd.config.json";

/** The page's script, built, and the files it loads: each by its path in the site. */
export interface Client {
  files: Map<string, Uint8Array>;
  /** The script's path in the site. */
  script: string;
}

/** The script's source, beside this module: `client.ts` here, `client.js` once built. */
const CLIENT = fileURLToPath(new URL(`./client${extname(fileURLToPath(import.meta.url))}`, import.meta.url));

/** What the blog in `dir` gives its posts: its packs, when it has a `compmd.config.json`. */
export async function packsOf(dir: string): Promise<Given> {
  if (!existsSync(join(dir, PACKS))) return {};
  const { request } = await loadApp(dir);
  return { packs: request.packs, ambient: request.ambient, policy: request.policy };
}

/** The files under `dir`, by their `/`-separated paths there, but Vite's own (`.vite/`). */
async function filesUnder(dir: string): Promise<Map<string, Uint8Array>> {
  const files = new Map<string, Uint8Array>();
  for (const entry of await readdir(dir, { recursive: true, withFileTypes: true })) {
    const path = relative(dir, join(entry.parentPath, entry.name)).split(sep).join("/");
    if (entry.isFile() && !path.startsWith(".vite/")) files.set(path, await readFile(join(entry.parentPath, entry.name)));
  }
  return files;
}

/**
 * Builds the page's script for the blog in `dir`, published at `base` (`/blog/`), with the sandbox's frames and the
 * blog's packs under `compmd/`.
 *
 * @throws BlogError when Vite's build fails, with what it said.
 */
export async function buildClient(dir: string, base: string): Promise<Client> {
  const out = await mkdtemp(join(tmpdir(), "cblo-client-"));
  try {
    await build({
      root: dir,
      base,
      configFile: false,
      logLevel: "warn",
      publicDir: false,
      plugins: [compmd()],
      build: { outDir: out, emptyOutDir: true, manifest: true, rolldownOptions: { input: { client: CLIENT } } },
    });
    const manifest = JSON.parse(await readFile(join(out, ".vite", "manifest.json"), "utf8")) as Record<string, { file: string; name?: string; isEntry?: boolean }>;
    const entry = Object.values(manifest).find(({ isEntry, name }) => isEntry === true && name === "client");
    if (!entry) throw new BlogError("the page's script wasn't built");
    return { files: await filesUnder(out), script: entry.file };
  } catch (error) {
    if (error instanceof BlogError) throw error;
    throw new BlogError(`the page's script didn't build: ${error instanceof Error ? error.message : String(error)}`);
  } finally {
    await rm(out, { recursive: true, force: true });
  }
}
