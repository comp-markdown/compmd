import { gzipSync } from "node:zlib";
import { describe, expect, it } from "vitest";
import { tarballOf } from "../test/local-registry.ts";
import { archive, header } from "../test/tar-archive.ts";
import { binaryFiles, gunzip, textFiles, untar } from "./tar.ts";

/** A cap no test's archive comes near. */
const ROOMY = 1_000_000;

describe("the tarball reader", () => {
  it("reads what pnpm pack wrote, a long path from the ustar prefix among it", async () => {
    const files = textFiles(untar(await gunzip(new Uint8Array(tarballOf("kit-charts-1.0.0.tgz")), ROOMY), ROOMY));
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
    expect(new TextDecoder().decode(await gunzip(zipped, 4))).toBe("text");
  });

  it("stops a gunzip that unzips past its cap, as a small tarball of a huge pack does", async () => {
    const bomb = new Uint8Array(gzipSync(Buffer.alloc(5_000_000)));
    expect(bomb.length).toBeLessThan(10_000);
    await expect(gunzip(bomb, 1_000_000)).rejects.toThrow("it unzips to more than 1000000 bytes");
  });

  it("refuses an entry whose size isn't a whole number of bytes, as a negative one that would read backwards", () => {
    for (const size of ["-0000002000", "1e3", "\x80\0\0\0\0\0\0\0\0\0\0\x01"]) {
      const tar = new Uint8Array(2048);
      tar.set(header("package/a.js", size, "0"));
      expect(() => untar(tar, ROOMY), JSON.stringify(size)).toThrow("a tar entry's size isn't a whole number of bytes");
    }
  });

  it("refuses an entry that runs past the archive's end", () => {
    const tar = new Uint8Array(1024);
    tar.set(header("package/a.js", 4096, "0"));
    expect(() => untar(tar, ROOMY)).toThrow("a tar entry runs past the tarball's end");
  });

  it("refuses files larger together than the size the registry declares", () => {
    const tar = archive(["package/a.js", "x".repeat(600)], ["package/b.js", "x".repeat(600)]);
    expect(untar(tar, 1200).size).toBe(2);
    expect(() => untar(tar, 1000)).toThrow("its files take more than the 1000 bytes its registry declares");
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
      ROOMY,
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
      ROOMY,
    );
    expect([...files.keys()]).toEqual(["kept.js"]);
  });

  it("keeps text files as text and the others as bytes, and stops at the end of the archive", () => {
    const tar = archive(["package/logo.png", new Uint8Array([0x89, 0x50, 0xff, 0xfe])], ["package/a.json", "{}"]);
    expect(textFiles(untar(tar, ROOMY))).toEqual({ "a.json": "{}" });
    expect(binaryFiles(untar(tar, ROOMY))).toEqual({ "logo.png": new Uint8Array([0x89, 0x50, 0xff, 0xfe]) });
    expect(untar(new Uint8Array(100), ROOMY)).toEqual(new Map());
  });
});
