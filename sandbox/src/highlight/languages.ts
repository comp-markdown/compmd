// The languages a page's code blocks are highlighted in: the popular ones, each by Shiki's name, with the fence names
// that mean it. Each grammar is a chunk of its own, loaded the first time a block of it shows; any other language
// stays plain.
import type { LanguageRegistration } from "@shikijs/core";

type Grammar = () => Promise<{ default: LanguageRegistration[] }>;

/** Each language, by Shiki's name: its grammar and the other fence names for it. */
const LANGUAGES: Record<string, [Grammar, ...string[]]> = {
  c: [() => import("@shikijs/langs/c")],
  cpp: [() => import("@shikijs/langs/cpp"), "c++"],
  csharp: [() => import("@shikijs/langs/csharp"), "c#", "cs"],
  java: [() => import("@shikijs/langs/java")],
  kotlin: [() => import("@shikijs/langs/kotlin"), "kt", "kts"],
  scala: [() => import("@shikijs/langs/scala")],
  swift: [() => import("@shikijs/langs/swift")],
  "objective-c": [() => import("@shikijs/langs/objective-c"), "objc"],
  go: [() => import("@shikijs/langs/go")],
  rust: [() => import("@shikijs/langs/rust"), "rs"],
  zig: [() => import("@shikijs/langs/zig")],
  dart: [() => import("@shikijs/langs/dart")],
  python: [() => import("@shikijs/langs/python"), "py"],
  ruby: [() => import("@shikijs/langs/ruby"), "rb"],
  php: [() => import("@shikijs/langs/php")],
  perl: [() => import("@shikijs/langs/perl")],
  lua: [() => import("@shikijs/langs/lua")],
  r: [() => import("@shikijs/langs/r")],
  julia: [() => import("@shikijs/langs/julia"), "jl"],
  powershell: [() => import("@shikijs/langs/powershell"), "ps", "ps1", "pwsh"],
  shellscript: [() => import("@shikijs/langs/shellscript"), "bash", "sh", "shell", "zsh"],
  bat: [() => import("@shikijs/langs/bat"), "batch", "cmd"],
  haskell: [() => import("@shikijs/langs/haskell"), "hs"],
  elixir: [() => import("@shikijs/langs/elixir")],
  erlang: [() => import("@shikijs/langs/erlang"), "erl"],
  clojure: [() => import("@shikijs/langs/clojure"), "clj"],
  fsharp: [() => import("@shikijs/langs/fsharp"), "f#", "fs"],
  ocaml: [() => import("@shikijs/langs/ocaml")],
  javascript: [() => import("@shikijs/langs/javascript"), "js", "cjs", "mjs"],
  typescript: [() => import("@shikijs/langs/typescript"), "ts", "cts", "mts"],
  jsx: [() => import("@shikijs/langs/jsx")],
  tsx: [() => import("@shikijs/langs/tsx")],
  html: [() => import("@shikijs/langs/html")],
  css: [() => import("@shikijs/langs/css")],
  scss: [() => import("@shikijs/langs/scss")],
  less: [() => import("@shikijs/langs/less")],
  svelte: [() => import("@shikijs/langs/svelte")],
  vue: [() => import("@shikijs/langs/vue")],
  astro: [() => import("@shikijs/langs/astro")],
  graphql: [() => import("@shikijs/langs/graphql"), "gql"],
  json: [() => import("@shikijs/langs/json")],
  jsonc: [() => import("@shikijs/langs/jsonc")],
  json5: [() => import("@shikijs/langs/json5")],
  yaml: [() => import("@shikijs/langs/yaml"), "yml"],
  toml: [() => import("@shikijs/langs/toml")],
  ini: [() => import("@shikijs/langs/ini"), "properties"],
  xml: [() => import("@shikijs/langs/xml")],
  csv: [() => import("@shikijs/langs/csv")],
  sql: [() => import("@shikijs/langs/sql")],
  proto: [() => import("@shikijs/langs/proto"), "protobuf"],
  docker: [() => import("@shikijs/langs/docker"), "dockerfile"],
  make: [() => import("@shikijs/langs/make"), "makefile"],
  cmake: [() => import("@shikijs/langs/cmake")],
  hcl: [() => import("@shikijs/langs/hcl")],
  nix: [() => import("@shikijs/langs/nix")],
  nginx: [() => import("@shikijs/langs/nginx")],
  diff: [() => import("@shikijs/langs/diff")],
  markdown: [() => import("@shikijs/langs/markdown"), "md"],
  latex: [() => import("@shikijs/langs/latex")],
  mermaid: [() => import("@shikijs/langs/mermaid"), "mmd"],
  matlab: [() => import("@shikijs/langs/matlab")],
  groovy: [() => import("@shikijs/langs/groovy")],
  solidity: [() => import("@shikijs/langs/solidity")],
};

/** Shiki's name for each fence name it highlights, case aside. */
const NAMES = new Map(Object.entries(LANGUAGES).flatMap(([name, [, ...aliases]]) => [name, ...aliases].map((alias) => [alias, name] as const)));

/** Shiki's name for the fence language `language`, or `undefined` for one that stays plain. */
export const languageOf = (language: string): string | undefined => NAMES.get(language.toLowerCase());

/** The grammar of a language by Shiki's name, with those it embeds. */
export const grammarOf = async (name: string): Promise<LanguageRegistration[]> => {
  const grammar = LANGUAGES[name]?.[0];
  return grammar ? (await grammar()).default : [];
};

/** Every language highlighted, by Shiki's name. */
export const LANGUAGE_NAMES: readonly string[] = Object.keys(LANGUAGES);
