// The release's platform packages (docs/releases.md, decision 9): from the binaries the release built, one package per
// platform, each with its `os` and `cpu`, and `@compmd/lsp`'s optional dependencies on them, at its version.
//
//   node npm-packages/lsp/src/write-platforms.ts <binaries> <out>
//
// `<binaries>` holds the binaries as the GitHub release names them (`compmd-lsp-<version>-<target>`); each package is
// written to `<out>/<os>-<cpu>`, and `npm-packages/lsp/package.json` gets the optional dependencies, in the release's
// checkout only.

import { chmodSync, copyFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { binaryOf, packageOf, PLATFORMS, releasedAs } from "./platforms.ts";

/** The launcher's `package.json`, as far as this reads and writes it. */
interface Launcher {
  version: string;
  optionalDependencies?: Record<string, string>;
  [key: string]: unknown;
}

/**
 * Writes a package per platform to `out` from the binaries in `binaries`, at the version of the launcher's
 * `package.json` at `launcher`, and adds them to it as its optional dependencies. Gives the folders written, in the
 * order they are staged.
 *
 * @throws Error when a platform's binary isn't in `binaries`.
 */
export function writePlatforms(binaries: string, out: string, launcher: string): string[] {
  const manifest = JSON.parse(readFileSync(launcher, "utf8")) as Launcher;
  const { version } = manifest;
  const licence = path.join(path.dirname(launcher), "LICENSE");
  const written = PLATFORMS.map((platform) => {
    const folder = path.join(out, `${platform.os}-${platform.cpu}`);
    mkdirSync(path.join(folder, "bin"), { recursive: true });
    const binary = path.join(folder, "bin", binaryOf(platform));
    copyFileSync(path.join(binaries, releasedAs(platform, version)), binary);
    chmodSync(binary, 0o755);
    copyFileSync(licence, path.join(folder, "LICENSE"));
    const name = packageOf(platform);
    const pkg = {
      name,
      version,
      description: `The CompMD language server's binary for ${platform.os} on ${platform.cpu}, which @compmd/lsp runs.`,
      homepage: "https://compmd.dev/",
      repository: { type: "git", url: "git+https://github.com/comp-markdown/compmd.git", directory: "npm-packages/lsp" },
      license: "MIT",
      os: [platform.os],
      cpu: [platform.cpu],
      files: ["bin"],
    };
    writeFileSync(path.join(folder, "package.json"), `${JSON.stringify(pkg, null, 2)}\n`);
    writeFileSync(path.join(folder, "README.md"), `# ${name}\n\nThe [CompMD](https://compmd.dev/) language server's binary for ${platform.os} on ${platform.cpu}. Install \`@compmd/lsp\`, which runs it.\n`);
    return folder;
  });
  manifest.optionalDependencies = Object.fromEntries(PLATFORMS.map((platform) => [packageOf(platform), version]));
  writeFileSync(launcher, `${JSON.stringify(manifest, null, 2)}\n`);
  return written;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const [binaries, out] = process.argv.slice(2);
  if (binaries === undefined || out === undefined) {
    process.stderr.write("usage: write-platforms.ts <binaries> <out>\n");
    process.exit(2);
  }
  const launcher = fileURLToPath(new URL("../package.json", import.meta.url));
  for (const folder of writePlatforms(binaries, out, launcher)) process.stdout.write(`${folder}\n`);
}
