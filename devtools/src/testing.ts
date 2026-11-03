// @compmd/devtools/testing: what a host's tests use to check its documents as the tools process them: the processing
// their worker runs, a diagnostic's fix as an edit, and the passages text-fragment links name.

export { editOf, fixOf, offsetOf, type Edit } from "./edit.ts";
export { findPassage, parseFragment, type Passage } from "./fragment.ts";
export { Processing, type Fetched, type ProcessingOptions, type Resolution } from "./processing.ts";
export type { FromWorker, ToWorker } from "./protocol.ts";
