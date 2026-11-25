// Where the binary for a platform is: in its package, installed beside this one.

import { fileURLToPath } from "node:url";
import { binaryOf, packageOf, platformOf } from "./platforms.ts";

/**
 * The path of the binary for `os` and `cpu` (Node's names), found with `resolve` (`import.meta.resolve`); `undefined`
 * when there is no binary for the platform, or its package isn't installed.
 */
export function binaryPath(os: string, cpu: string, resolve: (specifier: string) => string): string | undefined {
  const platform = platformOf(os, cpu);
  if (!platform) return undefined;
  try {
    return fileURLToPath(resolve(`${packageOf(platform)}/bin/${binaryOf(platform)}`));
  } catch {
    return undefined;
  }
}
