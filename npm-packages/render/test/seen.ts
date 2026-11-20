// The `compmd` prop a test component last got, so a test can call its `fail` after the component drew, or read its `fit`.
import type { Fit } from "../src/contract.ts";

/** What a component keeps of its `compmd` prop. */
export interface Seen {
  compmd?: { fail(message: string): void; fit?: Fit };
}

/** The Svelte `Plots` component's. */
export const seen: Seen = {};
