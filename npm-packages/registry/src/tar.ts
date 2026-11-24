// A pack's tarball, read (docs/resolving-registry-packs.md, decisions 6 and 11): a gzipped tar of the package under
// `package/`. Its regular files are kept, by their path in the package: those that are UTF-8 text for the processor,
// and the others as bytes, for the host to show. It is read only after its hash is checked, and every step is bounded
// (audit S10, S11): the gunzip by a cap, the entries by sizes that are whole numbers, and the files by the size their
// registry declares.

import { readCapped } from "./capped.ts";

const BLOCK = 512;
const PACKAGE = "package/";

/**
 * `bytes`, gunzipped with the platform's `DecompressionStream`.
 *
 * @param cap The most bytes it may unzip to.
 * @throws Error when it unzips to more than `cap` bytes, or isn't gzip.
 */
export async function gunzip(bytes: Uint8Array<ArrayBuffer>, cap: number): Promise<Uint8Array<ArrayBuffer>> {
  const stream = new Blob([bytes]).stream().pipeThrough(new DecompressionStream("gzip"));
  return readCapped(stream, cap, `it unzips to more than ${String(cap)} bytes`);
}

/** The text of a header field: up to its first NUL. */
function field(header: Uint8Array, start: number, length: number): string {
  const bytes = header.subarray(start, start + length);
  const end = bytes.indexOf(0);
  return new TextDecoder().decode(end === -1 ? bytes : bytes.subarray(0, end));
}

/**
 * An entry's size, its header's octal field: a whole number of bytes, `0` when the field is blank.
 *
 * @throws Error for a field that isn't octal digits, as a negative size or base-256 one is.
 */
function sizeOf(header: Uint8Array): number {
  const written = field(header, 124, 12).trim();
  if (!/^[0-7]*$/.test(written)) throw new Error(`a tar entry's size isn't a whole number of bytes: \`${written}\``);
  return Number.parseInt(written || "0", 8);
}

/** The `path` record of a PAX extended header (`<length> path=<value>\n`), if it has one. */
function paxPath(data: Uint8Array): string | undefined {
  const text = new TextDecoder().decode(data);
  for (const record of text.split("\n")) {
    const match = /^\d+ path=(.*)$/s.exec(record);
    if (match) return match[1];
  }
  return undefined;
}

/** The path of an entry in the package, or `undefined` for one outside `package/` or one that would leave it. */
function packagePath(name: string): string | undefined {
  if (!name.startsWith(PACKAGE)) return undefined;
  const path = name.slice(PACKAGE.length);
  const parts = path.split("/");
  const unsafe = path === "" || path.includes("\\") || parts.some((part) => part === ".." || part === "." || part === "");
  return unsafe ? undefined : path;
}

/**
 * The regular files of a tar archive, by their path in the package. Each entry's size is a whole number, so the
 * reading always moves on.
 *
 * @param maxSize The most bytes the package's files may take together: the unpacked size its registry declares.
 * @throws Error for an entry whose size isn't a whole number, one that runs past the archive's end, or files that take
 *   more than `maxSize` bytes.
 */
export function untar(archive: Uint8Array, maxSize: number): Map<string, Uint8Array> {
  const files = new Map<string, Uint8Array>();
  let offset = 0;
  let total = 0;
  let longName: string | undefined;
  while (offset + BLOCK <= archive.length) {
    const header = archive.subarray(offset, offset + BLOCK);
    if (header.every((byte) => byte === 0)) break;
    const size = sizeOf(header);
    const type = String.fromCharCode(header[156] ?? 0);
    if (offset + BLOCK + size > archive.length) throw new Error("a tar entry runs past the tarball's end");
    const data = archive.subarray(offset + BLOCK, offset + BLOCK + size);
    offset += BLOCK + Math.ceil(size / BLOCK) * BLOCK;
    if (type === "0" || type === "\0") {
      total += size;
      if (total > maxSize) throw new Error(`its files take more than the ${String(maxSize)} bytes its registry declares`);
    }
    if (type === "x") {
      longName = paxPath(data);
      continue;
    }
    if (type === "L") {
      longName = field(data, 0, data.length);
      continue;
    }
    const prefix = field(header, 345, 155);
    const name = longName ?? (prefix ? `${prefix}/${field(header, 0, 100)}` : field(header, 0, 100));
    longName = undefined;
    const path = packagePath(name);
    if ((type === "0" || type === "\0") && path !== undefined) files.set(path, data);
  }
  return files;
}

/**
 * `bytes` as text, or `undefined` when they aren't UTF-8: an image or a font, which the processor never reads. A
 * fatal decoder throws a `TypeError` for bytes that aren't; that is the only error dropped.
 */
function utf8(bytes: Uint8Array): string | undefined {
  try {
    return new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  } catch (error) {
    if (error instanceof TypeError) return undefined;
    throw error;
  }
}

/** The files that are UTF-8 text, as text: what a release in a processor's request holds. */
export function textFiles(files: Map<string, Uint8Array>): Record<string, string> {
  const text: Record<string, string> = {};
  for (const [path, bytes] of files) {
    const decoded = utf8(bytes);
    if (decoded !== undefined) text[path] = decoded;
  }
  return text;
}

/**
 * The files that aren't UTF-8 text, as bytes: an image or a font, which the processor never reads, but which a link or
 * an image from the pack's definitions names (spec §12.3), so the host shows it.
 */
export function binaryFiles(files: Map<string, Uint8Array>): Record<string, Uint8Array> {
  const binary: Record<string, Uint8Array> = {};
  for (const [path, bytes] of files) if (utf8(bytes) === undefined) binary[path] = bytes;
  return binary;
}
