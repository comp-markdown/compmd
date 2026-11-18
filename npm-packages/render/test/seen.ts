// The `compmd` prop a test component last got, so a test can call its `fail` after the component drew.

/** What a component keeps of its `compmd` prop. */
export interface Seen {
  compmd?: { fail(message: string): void };
}

/** The Svelte `Plots` component's. */
export const seen: Seen = {};
