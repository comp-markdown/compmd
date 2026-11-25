import { mkdtempSync, mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { describe, expect, it } from "vitest";
import { binaryPath } from "./binary.ts";
import { packageOf, PLATFORMS, platformOf, releasedAs } from "./platforms.ts";
import { writePlatforms } from "./write-platforms.ts";

describe("the platforms", () => {
  it("are macOS, Linux and Windows on arm64 and x64, each with its package and its released binary", () => {
    expect(PLATFORMS.map(packageOf)).toEqual([
      "@compmd/lsp-darwin-arm64",
      "@compmd/lsp-darwin-x64",
      "@compmd/lsp-linux-arm64",
      "@compmd/lsp-linux-x64",
      "@compmd/lsp-win32-arm64",
      "@compmd/lsp-win32-x64",
    ]);
    expect(PLATFORMS.map((platform) => releasedAs(platform, "0.11.0"))).toContain("compmd-lsp-0.11.0-x86_64-pc-windows-msvc.exe");
    expect(platformOf("linux", "x64")?.target).toBe("x86_64-unknown-linux-musl");
    expect(platformOf("freebsd", "x64")).toBeUndefined();
  });
});

describe("the binary", () => {
  it("is in the platform's package, where it is installed", () => {
    const resolve = (specifier: string) => pathToFileURL(path.join("/node_modules", specifier)).href;
    expect(binaryPath("darwin", "arm64", resolve)).toBe(path.join("/node_modules/@compmd/lsp-darwin-arm64/bin/compmd-lsp"));
    expect(binaryPath("win32", "x64", resolve)).toBe(path.join("/node_modules/@compmd/lsp-win32-x64/bin/compmd-lsp.exe"));
    expect(binaryPath("aix", "ppc64", resolve), "no binary for it").toBeUndefined();
    const missing = () => {
      throw new Error("not installed");
    };
    expect(binaryPath("linux", "x64", missing), "its package isn't installed").toBeUndefined();
  });
});

describe("the release's platform packages", () => {
  it("are written from the released binaries, at the launcher's version, which takes them as optional dependencies", () => {
    const work = mkdtempSync(path.join(tmpdir(), "compmd-lsp-"));
    const binaries = path.join(work, "binaries");
    mkdirSync(binaries);
    for (const platform of PLATFORMS) writeFileSync(path.join(binaries, releasedAs(platform, "1.2.3")), platform.target);
    const launcher = path.join(work, "launcher", "package.json");
    mkdirSync(path.dirname(launcher));
    writeFileSync(launcher, JSON.stringify({ name: "@compmd/lsp", version: "1.2.3" }));
    writeFileSync(path.join(work, "launcher", "LICENSE"), "MIT");
    const folders = writePlatforms(binaries, path.join(work, "out"), launcher);
    expect(folders.map((folder) => path.basename(folder))).toEqual(["darwin-arm64", "darwin-x64", "linux-arm64", "linux-x64", "win32-arm64", "win32-x64"]);
    const windows = JSON.parse(readFileSync(path.join(work, "out", "win32-x64", "package.json"), "utf8")) as Record<string, unknown>;
    expect([windows.name, windows.version, windows.os, windows.cpu]).toEqual(["@compmd/lsp-win32-x64", "1.2.3", ["win32"], ["x64"]]);
    expect(readFileSync(path.join(work, "out", "win32-x64", "bin", "compmd-lsp.exe"), "utf8")).toBe("x86_64-pc-windows-msvc");
    if (process.platform !== "win32") expect(statSync(path.join(work, "out", "linux-x64", "bin", "compmd-lsp")).mode & 0o111).toBe(0o111);
    expect(readFileSync(path.join(work, "out", "linux-x64", "LICENSE"), "utf8")).toBe("MIT");
    const written = JSON.parse(readFileSync(launcher, "utf8")) as { optionalDependencies: Record<string, string> };
    expect(written.optionalDependencies["@compmd/lsp-linux-arm64"]).toBe("1.2.3");
    expect(Object.keys(written.optionalDependencies)).toHaveLength(6);
  });

  it("need every platform's binary", () => {
    const work = mkdtempSync(path.join(tmpdir(), "compmd-lsp-"));
    writeFileSync(path.join(work, "package.json"), JSON.stringify({ version: "1.2.3" }));
    writeFileSync(path.join(work, "LICENSE"), "MIT");
    expect(() => writePlatforms(work, path.join(work, "out"), path.join(work, "package.json"))).toThrow(/compmd-lsp-1\.2\.3-aarch64-apple-darwin/);
  });
});
