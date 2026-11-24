// `cargo make approve-packs` (docs/releases.md, decision 8): what npm has staged of the packs, `@comp-markdown/`,
// approved as `approve-staged.ts` says. With `--dry-run` it lists them and approves nothing.
//
//   node tools/packs/approve-packs.ts [--dry-run]

import { runApproving } from "./approve-staged.ts";
import { SCOPE } from "./placeholder.ts";

/** The packs, as `approveStaged` approves them. */
export const PACKS = { scope: SCOPE, what: "the packs" };

if (import.meta.main) await runApproving(PACKS);
