// The sources a host's page adds to its content security policy (docs/hosting.md, decision 10): the frames from the
// origin, Shiki and Temml from the frames' folder in host mode, and the registry proxy with `fetch: "host"`. Never the
// packs' folder, nor the whole origin: a pack's code runs in frames only. No Node.

import type { ReadConfig } from "./config.ts";

/** A page's policy as directives, each with its sources, as `tauri.conf.json`'s `security.csp` takes it. */
export type Csp = Record<string, string[]>;

/**
 * The sources a host's page adds to its policy for the bundle served at `origin`.
 *
 * @param origin Where the bundle is served: a custom scheme's (`compmd://localhost`), a user-content domain, or an
 *   app's own folder (`https://app.example/compmd/`).
 * @param config The configuration (`readConfig`): with registries the host fetches from, the page reaches the proxy.
 * @example
 * cspFor("compmd://localhost", readConfig(json));
 * // { "frame-src": ["compmd://localhost/"], "script-src": ["compmd://localhost/frames/"], … }
 */
export function cspFor(origin: string, config?: Pick<ReadConfig, "registries" | "fetch">): Csp {
  const base = origin.replace(/\/*$/, "/");
  const frames = `${base}frames/`;
  const proxied = config !== undefined && config.fetch === "host" && Object.keys(config.registries).length > 0;
  return {
    "frame-src": [base],
    "script-src": [frames],
    "connect-src": [frames, ...(proxied ? [`${base}registry/`] : [])],
    "frame-ancestors": ["'none'"],
  };
}
