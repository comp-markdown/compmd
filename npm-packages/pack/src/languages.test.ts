// The publish check of a pack's content languages (the companion §2): `compmd.languages` in its `package.json`.
import { rm } from "node:fs/promises";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { documentOf, folder, write } from "../test/folders.ts";
import { checkPack } from "./check.ts";
import { readPack } from "./pack.ts";

let dir: string;

/** A pack whose component `line` renders the code blocks of `chart`, with `languages` in its manifest. */
const packWith = (languages: unknown) => ({
  "package.json": { name: "@kit/charts", version: "0.1.0", compmd: { document: "./index.comp.md", languages }, files: ["dist", "grammars", "index.comp.md"] },
  "index.comp.md": `${documentOf(["dist/line.js"])}\n\`\`\`compmd component=line signature\nparams: {}\nrenders: chart\n\`\`\`\n`,
  "src/line.ts": "export default () => undefined;",
  "dist/line.js": "export default () => undefined;",
  "grammars/chart.tmLanguage.json": { scopeName: "source.chart", patterns: [] },
  "grammars/chart.wasm": "\u0000asm\u0001\u0000\u0000\u0000",
  "grammars/broken.json": "{",
  "grammars/nameless.json": { patterns: [] },
  "secret/chart.json": { scopeName: "source.chart" },
});

const problems = async (languages: unknown) => {
  await write(dir, packWith(languages));
  return checkPack(await readPack(dir));
};

beforeEach(async () => {
  dir = await folder();
});

afterEach(async () => {
  await rm(dir, { recursive: true, force: true });
});

describe("a pack's content languages", () => {
  it("find nothing wrong with a language its renderer renders, with its grammars and names", async () => {
    expect(await problems([{ id: "chart", textmate: "grammars/chart.tmLanguage.json", treeSitter: "grammars/chart.wasm", aliases: ["charts"], languageId: "chart", extensions: [".chart"] }])).toEqual([]);
    expect(await problems(undefined)).toEqual([]);
  });

  it("find a language no renderer renders, and an entry without an id", async () => {
    expect(await problems([{ id: "mermaid" }, { textmate: "grammars/chart.tmLanguage.json" }, "chart"])).toEqual([
      "`compmd.languages[0]`'s `id` is mermaid, which none of the pack's renderers `renders` (they render chart)",
      "`compmd.languages[1]` needs an `id`, the language its renderer `renders`",
      "`compmd.languages[2]` needs an `id`, the language its renderer `renders`",
    ]);
    expect(await problems({ id: "chart" })).toEqual(["package.json's `compmd.languages` is a list of the pack's content languages"]);
  });

  it("find a grammar that isn't there, isn't published or isn't what it says", async () => {
    const entry = (textmate: unknown, treeSitter?: string) => [{ id: "chart", textmate, ...(treeSitter === undefined ? {} : { treeSitter }) }];
    expect(await problems(entry("grammars/gone.json"))).toEqual(["`compmd.languages[0]`'s `textmate`, grammars/gone.json, isn't there"]);
    expect(await problems(entry("secret/chart.json"))).toEqual(["`compmd.languages[0]`'s `textmate`, secret/chart.json, isn't published: add it to package.json's `files`"]);
    expect(await problems(entry("grammars/broken.json"))).toEqual(["`compmd.languages[0]`'s `textmate`, grammars/broken.json, isn't JSON"]);
    expect(await problems(entry("grammars/nameless.json"))).toEqual(["`compmd.languages[0]`'s `textmate`, grammars/nameless.json, has no `scopeName`, which a TextMate grammar names itself by"]);
    expect(await problems(entry(3))).toEqual(["`compmd.languages[0]`'s `textmate` is a path in the pack"]);
    expect(await problems(entry("grammars/chart.tmLanguage.json", "grammars/chart.tmLanguage.json"))).toEqual(["`compmd.languages[0]`'s `treeSitter`, grammars/chart.tmLanguage.json, isn't a WebAssembly module"]);
  });

  it("find names that aren't strings", async () => {
    expect(await problems([{ id: "chart", aliases: "charts", extensions: [1], languageId: 2 }])).toEqual([
      "`compmd.languages[0]`'s `aliases` is a list of strings",
      "`compmd.languages[0]`'s `extensions` is a list of strings",
      "`compmd.languages[0]`'s `languageId` is a string",
    ]);
  });
});
