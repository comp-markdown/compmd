// Where the bundle is served, as a host's adapter gives the page (docs/hosting.md, decision 9): `tauri-plugin-compmd`
// sets it with an initialization script, an Electron preload or a server's page the same way, so a page names none.

/** What a host's adapter sets on the page: `globalThis.__COMPMD__`. */
interface Given {
  origin?: unknown;
}

/** The bundle's origin the host's adapter gave the page (`globalThis.__COMPMD__.origin`), if it gave one. */
export function givenOrigin(): string | undefined {
  const given = (globalThis as { __COMPMD__?: Given }).__COMPMD__;
  return typeof given?.origin === "string" ? given.origin : undefined;
}
