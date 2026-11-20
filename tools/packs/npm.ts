// npm, pinned to a version with `npm trust` (11.10) and staged publishing (11.15), as the release workflows pin it,
// and run outside this repository, whose `devEngines` names pnpm (RELEASING.md, *What publishing 0.1.0 taught*).

import { spawnSync } from "node:child_process";
import { tmpdir } from "node:os";

export const NPM_VERSION = "11.21.1";

/** The command line that runs the pinned npm with `args`. */
export const npmCommand = (args: readonly string[]): string[] => ["npx", "--yes", `npm@${NPM_VERSION}`, ...args];

/** Runs npm with some arguments; throws when it fails. */
export type RunNpm = (args: readonly string[]) => void;

/**
 * Runs the pinned npm with `args` in a folder outside the repository. The terminal is npm's, so npm asks for the
 * author's one-time code itself, and keeps it for the few minutes npm's two-factor window allows.
 *
 * @throws Error when npm exits with anything but 0.
 */
export const runNpm: RunNpm = (args) => {
  const [command = "npx", ...rest] = npmCommand(args);
  const { status } = spawnSync(command, rest, { cwd: tmpdir(), stdio: "inherit" });
  if (status !== 0) throw new Error(`${npmCommand(args).join(" ")} failed, exit code ${String(status)}`);
};
