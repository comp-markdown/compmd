// The lint every TypeScript package here runs (audit R12): each package's eslint.config.js is
// `lintConfig(import.meta.dirname, [...its own ignores])`. Not named eslint.config.js, so ESLint never takes it for
// the config of a file outside a package.

import js from "@eslint/js";
import tseslint from "typescript-eslint";

/**
 * The lint of the package in `dir`: ESLint's recommended rules and typescript-eslint's strict type-checked ones,
 * leaving out `node_modules`, `dist` and `ignores`.
 *
 * @param {string} dir The package's folder, `import.meta.dirname` in its config.
 * @param {string[]} [ignores] What else it leaves out: built output, fixtures, packages of their own.
 * @param {...import("typescript-eslint").ConfigArray[number]} own The package's own configs, after the shared ones.
 * @example
 * // npm-packages/registry/eslint.config.js
 * export default lintConfig(import.meta.dirname, ["test/fixtures"]);
 */
export function lintConfig(dir, ignores = [], ...own) {
  return tseslint.config(
    { ignores: ["node_modules", "dist", ...ignores] },
    js.configs.recommended,
    ...tseslint.configs.strictTypeChecked,
    { languageOptions: { parserOptions: { projectService: true, tsconfigRootDir: dir } } },
    { files: ["*.js"], ...tseslint.configs.disableTypeChecked },
    // A test asserts on what it has just made; a missing one fails the test either way.
    { files: ["**/*.test.ts"], rules: { "@typescript-eslint/no-non-null-assertion": "off" } },
    ...own,
  );
}
