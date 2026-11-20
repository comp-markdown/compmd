// The tests' packs, shared by the tools' test files.

import type { Pack } from "./versions.ts";

/** A pack's files at `version`, written as the packs write them (an inline array in package.json). */
export function packAt(name: string, version: string, documentVersionOf = version): Pack {
  return {
    name,
    manifest: `{\n  "name": "@comp-markdown/${name}",\n  "version": "${version}",\n  "keywords": ["compmd", "compmd-pack"],\n  "devDependencies": { "x": "1.0.0" }\n}\n`,
    document: `---\nmd:\n  import:\n    - from: ./dist/${name}.js\n      names: [${name}]\n  export:\n    pack: "@comp-markdown/${name}"\n    version: "${documentVersionOf}"\n    names: [${name}]\n---\n\n# ${name}\n`,
  };
}
