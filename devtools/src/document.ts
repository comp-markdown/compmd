// What the tools open: a document as a processor's request, with the files it reads, which the tools show beside its
// text, read-only.

import type { Request } from "@compmd/processor";

/** A document the tools open. */
export interface DevtoolsDocument {
  /** The document as a processor's request: its path, its files (its own text among them), packs, data, inputs. */
  request: Request;
  /** What it reads besides itself, shown read-only beside its text: its files, its packs' files, its remote and host data. */
  files?: readonly SupportingFile[];
}

/** A file a document reads, by the name it is shown under. */
export interface SupportingFile {
  name: string;
  text: string;
}
