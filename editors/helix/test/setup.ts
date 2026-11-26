// What a writer's setup is once the entries are in, against local builds, in a temporary folder: languages.toml in
// Helix's config, the grammars and the queries in its runtime, and a workspace whose own languages.toml points at the
// local server. The check and the interactive session both start from it.

import { cpSync, mkdirSync, mkdtempSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

/** A Helix set up with the drafts: where its files are, and the environment it runs in. */
export interface HelixSetup {
  /** The temporary folder that holds the rest. */
  root: string;
  /** The workspace Helix runs in, with its `.helix/languages.toml`. */
  work: string;
  /** The process's environment, with Helix's runtime and config folders. */
  environment: NodeJS.ProcessEnv;
}

/** The drafts' folder, `editors/helix/`. */
export const drafts = dirname(dirname(fileURLToPath(import.meta.url)));

/**
 * Helix with the drafts, the parsers built in `grammars`, `server` as compmd-lsp, and `browser`, when given, as the
 * preview's browser; Helix's `config.toml` is `config`.
 */
export function helixSetup(grammars: string, server: string, browser?: string, config = ""): HelixSetup {
  const root = mkdtempSync(join(tmpdir(), "compmd-helix-"));
  const runtime = join(root, "runtime");
  mkdirSync(runtime);
  symlinkSync(grammars, join(runtime, "grammars"));
  symlinkSync(join(drafts, "queries"), join(runtime, "queries"));
  const configs = join(root, "config");
  mkdirSync(join(configs, "helix"), { recursive: true });
  cpSync(join(drafts, "languages.toml"), join(configs, "helix", "languages.toml"));
  writeFileSync(join(configs, "helix", "config.toml"), config);
  const work = join(root, "work");
  mkdirSync(join(work, ".helix"), { recursive: true });
  const settings = browser === undefined ? "" : `config = { compmd = { preview = { browser = ${JSON.stringify(browser)} } } }\n`;
  writeFileSync(join(work, ".helix", "languages.toml"), `[language-server.compmd-lsp]\ncommand = ${JSON.stringify(server)}\n${settings}`);
  return { root, work, environment: { ...process.env, HELIX_RUNTIME: runtime, XDG_CONFIG_HOME: configs } };
}
