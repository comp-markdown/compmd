// A pack's folder, read one way for the Vite plugin and the bundle (`@compmd/vite`) and for the pack command
// (`@compmd/pack`): its manifest and name, its document, what its documents name, confined to the pack, and what it
// publishes. Node only.

export { isPackPath, joinedInPack } from "./pack-folder/confined.ts";
export { documentOf, identityOf, manifestAt } from "./pack-folder/manifest.ts";
export { PackError } from "./pack-folder/pack-error.ts";
export { isSkipped, publishedFiles } from "./pack-folder/published.ts";
export { reachOf, type Reach } from "./pack-folder/reach.ts";
export { mdOf, referencesOf, type References } from "./pack-folder/references.ts";
