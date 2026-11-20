/**
 * The file `name` where the host serves the frame documents, `base`, full or relative to the page: a frame document,
 * or a module the host's page loads from there.
 *
 * @example
 * frameUrl("/compmd/frames/", "strict.html"); // "https://app.example/compmd/frames/strict.html"
 */
export const frameUrl = (base: string, name: "strict.html" | "permissive.html" | "highlight.js" | "math.js"): string => new URL(name, new URL(base, location.href)).href;
