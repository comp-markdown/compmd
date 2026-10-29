// A host-language component's code as a JavaScript module the frame can load: sucrase strips types and JSX, and
// Svelte's compiler compiles Svelte. Imports stay as the author wrote them, for the import map to resolve. Each
// compiler loads the first time a component needs it.

import type { HostComponent } from "@compmd/processor";

/** A component's module, with a Svelte component's CSS, which the frame adopts as a stylesheet. */
export interface Compiled {
  code: string;
  css?: string;
}

/** The runtimes the frame has: React's for JSX and TSX, Svelte 5's. */
const RUNTIMES: Partial<Record<HostComponent["language"], string[]>> = { jsx: ["react@19"], tsx: ["react@19"], svelte: ["svelte@5"] };

/**
 * `component`'s code as a module.
 *
 * @throws Error when the code doesn't compile, or names a runtime the frame hasn't (`preact@10`).
 */
export async function compile(component: HostComponent): Promise<Compiled> {
  const { name, language, runtime, code } = component;
  if (runtime && !RUNTIMES[language]?.includes(runtime)) throw new Error(`this editor has no ${runtime} to run \`${name}\``);
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
