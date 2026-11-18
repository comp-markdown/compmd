import { defineConfig } from "vitest/config";

// `respond` over the Fetch API, in Node, against the serving contract's cases the Rust crate shares.
export default defineConfig({ test: { environment: "node", include: ["src/**/*.test.ts"] } });
