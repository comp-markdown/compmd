import js from "@eslint/js";
import tseslint from "typescript-eslint";

export default tseslint.config(
  { ignores: ["node_modules", "dist", "test/.frames"] },
  js.configs.recommended,
  ...tseslint.configs.strictTypeChecked,
  {
    languageOptions: {
      parserOptions: { projectService: true, tsconfigRootDir: import.meta.dirname },
    },
  },
  { files: ["*.js"], ...tseslint.configs.disableTypeChecked },
  // A test asserts on what it has just made; a missing one fails the test either way.
  { files: ["**/*.test.ts"], rules: { "@typescript-eslint/no-non-null-assertion": "off" } },
);
