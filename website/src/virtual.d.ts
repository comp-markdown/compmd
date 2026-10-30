declare module "virtual:snapshots" {
  import type { Snapshot } from "../build/snapshots.ts";

  /** The documents in `content/`, by name. */
  const snapshots: Record<"hero" | "components" | "data" | "inputs" | "layout" | "host" | "degrade" | "plain", Snapshot>;
  export default snapshots;

  /** The other files in `content/`, by name, which the documents read. */
  export const files: Record<string, string>;
}
