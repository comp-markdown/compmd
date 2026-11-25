// A component reader as the processor's module calls it: the props as JSON text, as every value crosses into it.

import type { ComponentReader } from "./index.ts";

/** `reader`, giving its props as JSON text; `undefined` without one. */
export function readerJson(reader: ComponentReader | undefined): ((language: string, code: string) => string | undefined) | undefined {
  if (!reader) return undefined;
  return (language, code) => {
    const props = reader(language, code);
    return props === undefined ? undefined : JSON.stringify(props);
  };
}
