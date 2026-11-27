import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { crateDirectory, crateFiles, crateIndexUrl, hasCrate, isCrateName } from "./crate-placeholder.ts";
import type { Fetch } from "./registry.ts";

describe("a crate's placeholder", () => {
  const files = crateFiles("compmd-lsp", "MIT License\n");

  it("is 0.0.0 of the crate, with an empty library", () => {
    assert.deepEqual(Object.keys(files).sort(), ["Cargo.toml", "LICENSE", "README.md", "src/lib.rs"]);
    assert.match(files["Cargo.toml"] ?? "", /^\[package\]\nname = "compmd-lsp"\nversion = "0\.0\.0"\n/);
    assert.match(files["src/lib.rs"] ?? "", /^\/\/! A placeholder with no code/);
  });

  it("is MIT, names the repository, and points to the crate's folder", () => {
    assert.match(files["Cargo.toml"] ?? "", /\nlicense = "MIT"\nrepository = "https:\/\/github.com\/comp-markdown\/compmd"\n/);
    assert.equal(files.LICENSE, "MIT License\n");
    assert.match(files["README.md"] ?? "", /tree\/main\/crates\/lsp\)/);
  });

  it("knows each crate's folder", () => {
    assert.equal(crateDirectory("compmd-project"), "crates/project");
    assert.equal(crateDirectory("compmd"), "crates/processor");
  });
});

describe("a crate's name", () => {
  it("is compmd, or compmd- and lowercase words", () => {
    for (const name of ["compmd", "compmd-lsp", "compmd-components"]) assert.equal(isCrateName(name), true, name);
    for (const name of ["", "lsp", "compmd_lsp", "compmd-", "Compmd-lsp", "@compmd/lsp", "tauri-plugin-compmd"]) assert.equal(isCrateName(name), false, name);
  });
});

describe("whether crates.io has a crate", () => {
  const answering =
    (status: number): Fetch =>
    () =>
      Promise.resolve({ status, ok: status === 200, json: () => Promise.resolve({}) });

  it("asks the sparse index, at the crate's path", () => {
    assert.equal(crateIndexUrl("compmd-lsp"), "https://index.crates.io/co/mp/compmd-lsp");
  });

  it("is yes for a record, no for not found, and an error for anything else", async () => {
    assert.equal(await hasCrate("compmd", answering(200)), true);
    assert.equal(await hasCrate("compmd-lsp", answering(404)), false);
    await assert.rejects(hasCrate("compmd-lsp", answering(503)), /answered 503 for compmd-lsp/);
  });
});
