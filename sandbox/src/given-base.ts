// The frames' `base` a host's app gives the page itself, so a page needs to name none: `tauri-plugin-compmd` sets it
// with its initialization script (docs/tauri-plugin.md, decision 5).

/** What a host's app sets on the page: `globalThis.__COMPMD__`. */
interface Given {
  base?: unknown;
}

/** The frames' `base` the host's app gave the page (`globalThis.__COMPMD__.base`), if it gave one. */
export function givenBase(): string | undefined {
  const given = (globalThis as { __COMPMD__?: Given }).__COMPMD__;
  return typeof given?.base === "string" ? given.base : undefined;
}
