// The component reader as WebAssembly (`@compmd/processor/components`): what a JSX, TSX or Svelte component takes,
// read from its source, for component analysis (W104). A module of its own, so a page that only renders never loads
// the parser: give its `readProps` to `process` or an open document as `componentReader`.

import { readProps as readPropsJson } from "../pkg/components/compmd_components_wasm.js";
import type { ComponentReader, Props } from "./index.ts";

export { default as init, initSync } from "../pkg/components/compmd_components_wasm.js";

/**
 * What the component of `language` (`jsx`, `tsx`, `svelte`) whose source is `code` takes: a React component's default
 * export's first parameter, a Svelte 5 component's `$props()`, each prop with its type where TypeScript states one.
 * `undefined` when its props can't be read, which leaves the component unchecked. The module must be loaded first,
 * with {@link init} or {@link initSync}.
 *
 * @example
 * readProps("jsx", "export default function Badge({ label }) { return null; }"); // { props: [{ name: "label" }], rest: false }
 */
export const readProps: ComponentReader = (language: string, code: string): Props | undefined => {
  const json = readPropsJson(language, code);
  return json === undefined ? undefined : (JSON.parse(json) as Props);
};
