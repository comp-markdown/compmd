// The libraries the page shares through its import map: compiled components import them as their authors write
// them, and the editor's own code does too, so there is one React and one Svelte runtime.

export interface Library {
  /** The npm package. */
  name: string;
  /** The import names components and the editor use. */
  imports: string[];
  /** Whether it is CommonJS, so its entries list its named exports. */
  commonjs?: boolean;
}

export const LIBRARIES: Library[] = [
  { name: "react", imports: ["react", "react/jsx-runtime"], commonjs: true },
  { name: "react-dom", imports: ["react-dom", "react-dom/client"], commonjs: true },
  {
    name: "svelte",
    imports: [
      "svelte",
      "svelte/internal/client",
      "svelte/internal/disclose-version",
      "svelte/internal/flags/legacy",
      "svelte/internal/flags/async",
      "svelte/reactivity",
      "svelte/store",
      "svelte/transition",
      "svelte/easing",
      "svelte/motion",
      "svelte/animate",
      "svelte/events",
      "svelte/attachments",
    ],
  },
  // For components that draw with it; the page loads it only when one imports it.
  { name: "d3", imports: ["d3"] },
];

/** Every import name the import map resolves. */
export const SHARED = new Set(LIBRARIES.flatMap((library) => library.imports));

/** The npm package of an import name. */
export const packageOf = (specifier: string): string => (specifier.startsWith("@") ? specifier.split("/").slice(0, 2).join("/") : (specifier.split("/")[0] ?? specifier));
