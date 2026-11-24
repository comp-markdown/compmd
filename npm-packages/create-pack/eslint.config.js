import { lintConfig } from "../../eslint.shared.js";

// A library pack's entry re-exports the library it is made for, and a wrapped pack's glue imports the package it
// wraps, which only a written pack has: the test that wraps a package checks the glue there.
export default lintConfig(import.meta.dirname, ["test/.out", "template/library/src/index.ts", "template/wrap"]);
