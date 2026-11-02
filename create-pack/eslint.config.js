import js from "@eslint/js";
import tseslint from "typescript-eslint";

export default tseslint.config(
  // A library pack's entry re-exports the library it is made for, which only a written pack has.
  { ignores: ["node_modules", "dist", "test/.out", "template/library/src/index.ts"] },
  js.configs.recommended,
  ...tseslint.configs.strictTypeChecked,
  {
    languageOptions: {
      parserOptions: { projectService: true, tsconfigRootDir: import.meta.dirname },
    },
  },
  { files: ["*.js", "template/**/*.js"], ...tseslint.configs.disableTypeChecked },
  // A written pack's own scripts run in Node.
  { files: ["template/**/scripts/*.js"], languageOptions: { globals: { console: "readonly", process: "readonly" } } },
  // A test asserts on what it has just made; a missing one fails the test either way.
  { files: ["**/*.test.ts"], rules: { "@typescript-eslint/no-non-null-assertion": "off" } },
);
