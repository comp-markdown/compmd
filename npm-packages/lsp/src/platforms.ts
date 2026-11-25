// The platforms the language server is released for (docs/releases.md, decision 9): each with Node's names for it,
// its Rust target and its npm package.

/** A platform `compmd-lsp` is built for. */
export interface Platform {
  /** Node's `process.platform`. */
  os: "darwin" | "linux" | "win32";
  /** Node's `process.arch`. */
  cpu: "arm64" | "x64";
  /** The Rust target it is built for. */
  target: string;
}

/** Every platform, in the order the release lists them. */
export const PLATFORMS: readonly Platform[] = [
  { os: "darwin", cpu: "arm64", target: "aarch64-apple-darwin" },
  { os: "darwin", cpu: "x64", target: "x86_64-apple-darwin" },
  { os: "linux", cpu: "arm64", target: "aarch64-unknown-linux-musl" },
  { os: "linux", cpu: "x64", target: "x86_64-unknown-linux-musl" },
  { os: "win32", cpu: "arm64", target: "aarch64-pc-windows-msvc" },
  { os: "win32", cpu: "x64", target: "x86_64-pc-windows-msvc" },
];

/** The npm package that holds `platform`'s binary: `@compmd/lsp-darwin-arm64`. */
export const packageOf = ({ os, cpu }: Platform): string => `@compmd/lsp-${os}-${cpu}`;

/** The binary's file name on `platform`. */
export const binaryOf = ({ os }: Platform): string => (os === "win32" ? "compmd-lsp.exe" : "compmd-lsp");

/** The file the GitHub release names `platform`'s binary of `version`: `compmd-lsp-0.11.0-aarch64-apple-darwin`. */
export const releasedAs = (platform: Platform, version: string): string =>
  `compmd-lsp-${version}-${platform.target}${platform.os === "win32" ? ".exe" : ""}`;

/** The platform Node runs on, `undefined` for one with no binary. */
export const platformOf = (os: string, cpu: string): Platform | undefined =>
  PLATFORMS.find((platform) => platform.os === os && platform.cpu === cpu);
