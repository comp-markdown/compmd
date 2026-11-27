import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { publishArgs, trustArgs } from "./pack-placeholders.ts";
import { cargoPublishArgs, implementationPlaceholder, isImplementationPackage, releasePlaceholders, type Steps } from "./release-placeholders.ts";

/** Steps that record what they were asked, with the registries having `published`. */
function stubs(published: readonly string[]) {
  const done: string[] = [];
  const log: string[] = [];
  const written: Record<string, string>[] = [];
  const steps: Steps = {
    hasPackage: (name) => Promise.resolve(published.includes(name)),
    hasCrate: (name) => Promise.resolve(published.includes(name)),
    write(files) {
      written.push(files);
      return `/tmp/placeholder-${String(written.length)}`;
    },
    npm: (args) => {
      done.push(`npm ${args.join(" ")}`);
    },
    cargo: (args, folder) => {
      done.push(`cargo ${args.join(" ")} in ${folder}`);
    },
    pause: () => {
      done.push("pause");
      return Promise.resolve();
    },
    log: (line) => {
      log.push(line);
    },
  };
  return { steps, done, log, written };
}

describe("cargo make release-placeholders", () => {
  it("publishes an npm package's placeholder and sets its publisher to release.yml, and a crate's with cargo", async () => {
    const { steps, done, log, written } = stubs(["@compmd/lsp-linux-x64", "compmd-project"]);
    const names = ["@compmd/lsp", "@compmd/lsp-linux-x64", "@compmd/lsp-darwin-arm64", "compmd-project", "compmd-lsp"];
    assert.equal(await releasePlaceholders(names, "MIT", { dryRun: false }, steps), 0);
    assert.deepEqual(done, [
      `npm ${publishArgs("/tmp/placeholder-1").join(" ")}`,
      "pause",
      `npm ${trustArgs("@compmd/lsp", "release.yml").join(" ")}`,
      "pause",
      `npm ${publishArgs("/tmp/placeholder-2").join(" ")}`,
      "pause",
      `npm ${trustArgs("@compmd/lsp-darwin-arm64", "release.yml").join(" ")}`,
      `cargo ${cargoPublishArgs.join(" ")} in /tmp/placeholder-3`,
    ]);
    assert.equal(written[2]?.["src/lib.rs"]?.startsWith("//!"), true, "the crate's files");
    assert.deepEqual(
      log.filter((line) => line.startsWith("compmd-lsp: ")),
      [
        "compmd-lsp: 0.0.0 published",
        "compmd-lsp: set its trusted publisher on https://crates.io/crates/compmd-lsp/settings, Trusted Publishing, GitHub: the owner comp-markdown, the repository compmd, the workflow release.yml, the environment crates-io",
      ],
    );
    assert.ok(log.includes("@compmd/lsp-linux-x64: npm has it already, left as it is"));
    assert.ok(log.includes("compmd-project: crates.io has it already, left as it is"));
    assert.match(log.at(-1) ?? "", /within 2 days/);
  });

  it("with --dry-run, says what it would run, and does nothing", async () => {
    const { steps, done, log, written } = stubs([]);
    assert.equal(await releasePlaceholders(["@compmd/lsp", "compmd-lsp"], "MIT", { dryRun: true }, steps), 0);
    assert.deepEqual(done, []);
    assert.deepEqual(written, []);
    assert.deepEqual(log, [
      "@compmd/lsp: would publish 0.0.0 (package.json, README.md, LICENSE), then set its trusted publisher:",
      "  npx --yes npm@11.21.1 publish <a new folder> --access public",
      "  npx --yes npm@11.21.1 trust github @compmd/lsp --file release.yml --repository comp-markdown/compmd --allow-stage-publish --yes",
      "compmd-lsp: would publish 0.0.0 (Cargo.toml, src/lib.rs, README.md, LICENSE) from a new folder: cargo publish --allow-dirty",
      "  then set its trusted publisher on https://crates.io/crates/compmd-lsp/settings, Trusted Publishing, GitHub: the owner comp-markdown, the repository compmd, the workflow release.yml, the environment crates-io",
    ]);
  });

  it("says nothing of 2 days when the registries had them all", async () => {
    const { steps, log } = stubs(["@compmd/lsp"]);
    assert.equal(await releasePlaceholders(["@compmd/lsp"], "MIT", { dryRun: false }, steps), 0);
    assert.doesNotMatch(log.join("\n"), /2 days/);
  });

  it("does nothing for a name that is neither an @compmd/ package nor a compmd crate, or none", async () => {
    for (const names of [["@compmd/lsp", "@comp-markdown/card"], ["lsp"], []]) {
      const { steps, done } = stubs([]);
      assert.equal(await releasePlaceholders(names, "MIT", { dryRun: false }, steps), 2);
      assert.deepEqual(done, []);
    }
  });

  it("stops at a failure, the names before it done", async () => {
    const { steps, done } = stubs([]);
    steps.cargo = () => {
      throw new Error("cargo failed");
    };
    await assert.rejects(releasePlaceholders(["@compmd/lsp", "compmd-lsp", "compmd-project"], "MIT", { dryRun: false }, steps), /cargo failed/);
    assert.equal(done.length, 3, "the npm package's publish, pause and trust");
  });
});

describe("an implementation package's placeholder", () => {
  it("is in the package's folder, the language server's platform packages in its", () => {
    assert.deepEqual(implementationPlaceholder("@compmd/lsp-win32-arm64"), { name: "@compmd/lsp-win32-arm64", directory: "npm-packages/lsp", keywords: ["compmd"] });
    assert.equal(implementationPlaceholder("@compmd/lsp").directory, "npm-packages/lsp");
    assert.equal(implementationPlaceholder("@compmd/blog").directory, "npm-packages/blog");
  });

  it("is of an @compmd/ name only", () => {
    assert.equal(isImplementationPackage("@compmd/lsp-darwin-x64"), true);
    for (const name of ["@comp-markdown/card", "@compmd/", "@compmd/Lsp", "compmd-lsp"]) assert.equal(isImplementationPackage(name), false, name);
  });
});
