// A component's imports of library packs, written as the URLs of the modules the frame made for them
// (docs/sharing-library-packs.md, decision 6). The parser is es-module-lexer's plain JavaScript build: the strict
// policy has no WebAssembly. It loads the first time code imports a library pack.

/** What es-module-lexer gives of an import: its specifier, unescaped, where it is, and whether it is `import()`. */
interface Import {
  n?: string | undefined;
  s: number;
  e: number;
  d: number;
}

/**
 * `text` with each import of a name in `urls` (static, `export … from`, and `import()` of a string literal) written
 * as that name's URL; every other import as it was.
 *
 * @throws Error when `text` isn't a module es-module-lexer can read.
 * @example
 * await withImports('import * as d3 from "@kit/d3";', { "@kit/d3": "blob:null/1f…" });
 * // 'import * as d3 from "blob:null/1f…";'
 */
export async function withImports(text: string, urls: Readonly<Record<string, string>>): Promise<string> {
  const { parse } = await import("es-module-lexer/js");
  const [imports] = parse(text) as unknown as [readonly Import[]];
  let written = "";
  let at = 0;
  for (const { n: name, s: start, e: end, d: dynamic } of imports) {
    if (name === undefined || !Object.hasOwn(urls, name)) continue;
    const url = urls[name] as string;
    // A static import's range is the specifier inside its quotes; `import()`'s is the string, quotes and all.
    written += text.slice(at, start) + (dynamic >= 0 ? JSON.stringify(url) : url);
    at = end;
  }
  return written + text.slice(at);
}
