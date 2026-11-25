import { lintConfig } from "../eslint.shared.js";

// `playground/packs/` holds packages of their own, each with its own build and tests; `playground/examples/` are
// documents' components, which the playground compiles as it renders them.
export default lintConfig(import.meta.dirname, ["dist-server", "public", "content", "playground/examples", "playground/packs", "playground/test/.out"]);
