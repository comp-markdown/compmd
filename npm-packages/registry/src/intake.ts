// A version's tarball taken in (audit A9): one bounded pipeline from the download to the files. The download is read up
// to the host's size cap, hashed, gunzipped up to the cap, and read as a tar whose files may take no more than the
// unpacked size the registry declares. Nothing is held whole before it is found too large.

import { integrityOf } from "./integrity.ts";
import type { Registry, VersionInfo } from "./registry.ts";
import { gunzip, untar } from "./tar.ts";

/**
 * The files of `info`'s tarball, by their path in the package, once its bytes match `integrity`.
 *
 * @param maxSize The host's size cap, in bytes: the most the tarball and the tar it unzips to may take, and the files
 *   when the registry declares no unpacked size.
 * @returns `undefined` when the tarball doesn't match `integrity`.
 * @throws Error saying which step refused it: a download, a gunzip or files larger than allowed, or a malformed tar.
 */
export async function intake(registry: Registry, info: VersionInfo, integrity: string, maxSize: number): Promise<Map<string, Uint8Array> | undefined> {
  const tarball = await registry.tarball(info, maxSize);
  // A host's own registry may read past the cap; it is refused all the same.
  if (tarball.length > maxSize) throw new Error(`${info.tarball} is larger than ${String(maxSize)} bytes`);
  if ((await integrityOf(tarball)) !== integrity) return undefined;
  return untar(await gunzip(tarball, maxSize), Math.min(info.unpackedSize ?? maxSize, maxSize));
}
