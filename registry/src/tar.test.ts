import { gzipSync } from "node:zlib";
import { describe, expect, it } from "vitest";
import { tarballOf } from "../test/local-registry.ts";
import { gunzip, textFiles, untar } from "./tar.ts";

/** A tar header for `name`, `size` bytes of type `type`, with `prefix` in the ustar prefix field. */
function header(name: string, size: number, type: string, prefix = ""): Uint8Array {
  const block = new Uint8Array(512);
  const put = (text: string, at: number) => {
    block.set(new TextEncoder().encode(text), at);
  };
  put(name, 0);
  put(size.toString(8).padStart(11, "0"), 124);
  put(type, 156);
  put("ustar", 257);
  put(prefix, 345);
  return block;
}

/** An archive of `entries`, each a header and its data, padded to blocks, then the two empty blocks that end it. */
function archive(...entries: [name: string, data: string | Uint8Array, type?: string, prefix?: string][]): Uint8Array {
  const parts = entries.flatMap(([name, data, type = "0", prefix]) => {
    const bytes = typeof data === "string" ? new TextEncoder().encode(data) : data;
    const padded = new Uint8Array(Math.ceil(bytes.length / 512) * 512);
    padded.set(bytes);
    return [header(name, bytes.length, type, prefix), padded];
  });
  const all = [...parts, new Uint8Array(1024)];
  const out = new Uint8Array(all.reduce((length, part) => length + part.length, 0));
  let at = 0;
  for (const part of all) {
    out.set(part, at);
    at += part.length;
  }
  return out;
}

describe("the tarball reader", () => {
  it("reads what pnpm pack wrote, a long path from the ustar prefix among it", async () => {
    const files = textFiles(untar(await gunzip(new Uint8Array(tarballOf("kit-charts-1.0.0.tgz")))));
    expect(Object.keys(files).sort()).toEqual([
      "data/a-folder-name-long-enough-for-the-ustar-prefix-field/and-another-one-to-pass-a-hundred-characters/points.json",
      "dist/chart.js",
      "index.comp.md",
      "package.json",
    ]);
    expect(JSON.parse(files["package.json"]!)).toMatchObject({ name: "@kit/charts", version: "1.0.0" });
  });

  it("gunzips with the platform's DecompressionStream", async () => {
    const zipped = new Uint8Array(gzipSync(Buffer.from("text")));
    expect(new TextDecoder().decode(await gunzip(zipped))).toBe("text");
  });

  it("takes long names from PAX and GNU headers", () => {
    const long = `package/${"a".repeat(120)}.md`;
    const files = untar(
      archive(
        ["PaxHeader", `${String(long.length + 15)} path=${long}\n`, "x"],
        ["short-name", "pax"],
        ["././@LongLink", `package/${"b".repeat(120)}.md\0`, "L"],
        ["short-name", "gnu"],
        ["index.comp.md", "prefixed", "0", "package"],
      ),
    );
    expect(new TextDecoder().decode(files.get(`${"a".repeat(120)}.md`))).toBe("pax");
    expect(new TextDecoder().decode(files.get(`${"b".repeat(120)}.md`))).toBe("gnu");
    expect(new TextDecoder().decode(files.get("index.comp.md"))).toBe("prefixed");
  });

  it("drops links, folders, paths that leave the package and entries outside it", () => {
    const files = untar(
      archive(
        ["package/link.js", "", "2"],
        ["package/dist/", "", "5"],
        ["package/../escape.js", "x"],
        ["package/a//b.js", "x"],
        ["package/a\\b.js", "x"],
        ["package/./a.js", "x"],
        ["node/index.js", "x"],
        ["package/kept.js", "kept"],
      ),
    );
    expect([...files.keys()]).toEqual(["kept.js"]);
  });

  it("keeps only text files, and stops at the end of the archive", () => {
    const tar = archive(["package/logo.png", new Uint8Array([0x89, 0x50, 0xff, 0xfe])], ["package/a.json", "{}"]);
    expect(textFiles(untar(tar))).toEqual({ "a.json": "{}" });
    expect(untar(new Uint8Array(100))).toEqual(new Map());
  });
});
