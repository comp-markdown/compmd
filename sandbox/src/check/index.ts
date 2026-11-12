// @compmd/sandbox/check: the check every host runs in its real browser or webview (docs/hosting.md, decision 11), and
// the serving contract's headers it checks.
export { runCheck, type CheckedPack, type CheckOptions, type CheckReport } from "./check.ts";
export { CONTRACT_HEADERS } from "./contract.ts";
