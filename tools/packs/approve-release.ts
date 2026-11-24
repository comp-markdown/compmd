// `cargo make approve-release` (RELEASING.md): what npm has staged of a release's packages, `@compmd/`, approved as
// `approve-staged.ts` says, in the order the release workflow staged them, so a package is never on npm before those
// it depends on. With `--dry-run` it lists them and approves nothing.
//
//   node tools/packs/approve-release.ts [--dry-run]

import { runApproving } from "./approve-staged.ts";

/** A release's npm packages, as `approveStaged` approves them. */
export const RELEASE = { scope: "@compmd", what: "the release" };

if (import.meta.main) await runApproving(RELEASE);
