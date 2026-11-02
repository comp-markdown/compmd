// What the protocol's tests share: waiting for posted messages, their types, and a use that binds an input.

import type { ExpandedComponent } from "@compmd/render";

/** Lets the messages posted so far arrive. */
export const settle = () =>
  new Promise<void>((resolve) => {
    setImmediate(() => {
      setImmediate(resolve);
    });
  });

/** The types of `messages`. */
export const types = (messages: unknown[]) => messages.map((message) => (message as { type: string }).type);

/** A use of `chart` whose `selected` is bound to the input `region`. */
export const chart: ExpandedComponent = {
  type: "componentLeaf",
  name: "chart",
  params: [
    { key: "rows", breakpoint: null, value: [] },
    { key: "selected", breakpoint: null, value: "eu", input: "region" },
  ],
};
