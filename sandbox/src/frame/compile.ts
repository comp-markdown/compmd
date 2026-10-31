// A host-language component's source as a JavaScript module a frame can load: sucrase strips types and JSX, and
// Svelte's compiler compiles Svelte. Imports stay as the author wrote them, for the import map to resolve. Each
// compiler loads the first time a component needs it. The compile frame runs this, and no component.

import type { CompileRequest } from "../protocol/messages.ts";

/** A component's module, with a Svelte component's CSS, which the frame adopts as a stylesheet. */
export interface Compiled {
  code: string;
  css?: string;
}

/** The runtimes the frames have: React's for JSX and TSX, Svelte 5's. */
const RUNTIMES: Partial<Record<CompileRequest["language"], string[]>> = { jsx: ["react@19"], tsx: ["react@19"], svelte: ["svelte@5"] };

/**
 * `request`'s source as a module.
 *
 * @throws Error when the code doesn't compile, or names a runtime the frames haven't (`preact@10`).
 */
export async function compile(request: CompileRequest): Promise<Compiled> {
  const { name, language, runtime, code } = request;
  if (runtime && !RUNTIMES[language]?.includes(runtime)) throw new Error(`this sandbox has no ${runtime} to run \`${name}\``);
  if (language === "svelte") {
    const svelte = await import("svelte/compiler");
    const compiled = svelte.compile(code, { filename: `${name}.svelte`, generate: "client", css: "external" });
    return { code: compiled.js.code, ...(compiled.css ? { css: compiled.css.code } : {}) };
  }
  if (language === "js") return { code };
  const { transform } = await import("sucrase");
  const transforms: ("jsx" | "typescript")[] = language === "jsx" ? ["jsx"] : language === "tsx" ? ["jsx", "typescript"] : ["typescript"];
  return { code: transform(code, { transforms, jsxRuntime: "automatic", production: true, filePath: `${name}.${language}` }).code };
}
