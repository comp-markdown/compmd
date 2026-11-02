// A pack's tarball, read (docs/resolving-registry-packs.md, decisions 6 and 11): a gzipped tar of the package under
// `package/`. Only what the processor can use is kept: regular files, by their path in the package, that are UTF-8
// text. It is read only after its hash is checked.

const BLOCK = 512;
const PACKAGE = "package/";

/** `bytes`, gunzipped with the platform's `DecompressionStream`. */
export async function gunzip(bytes: Uint8Array<ArrayBuffer>): Promise<Uint8Array<ArrayBuffer>> {
  const stream = new Blob([bytes]).stream().pipeThrough(new DecompressionStream("gzip"));
  return new Uint8Array(await new Response(stream).arrayBuffer());
}

/** The text of a header field: up to its first NUL. */
function field(header: Uint8Array, start: number, length: number): string {
  const bytes = header.subarray(start, start + length);
  const end = bytes.indexOf(0);
  return new TextDecoder().decode(end === -1 ? bytes : bytes.subarray(0, end));
}

/** A number in a header field, written in octal. */
const octal = (header: Uint8Array, start: number, length: number): number => Number.parseInt(field(header, start, length).trim() || "0", 8);

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

/** The regular files of a tar archive, by their path in the package. */
export function untar(archive: Uint8Array): Map<string, Uint8Array> {
  const files = new Map<string, Uint8Array>();
  let offset = 0;
  let longName: string | undefined;
  while (offset + BLOCK <= archive.length) {
    const header = archive.subarray(offset, offset + BLOCK);
    if (header.every((byte) => byte === 0)) break;
    const size = octal(header, 124, 12);
    const type = String.fromCharCode(header[156] ?? 0);
    const data = archive.subarray(offset + BLOCK, offset + BLOCK + size);
    offset += BLOCK + Math.ceil(size / BLOCK) * BLOCK;
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

/** The files that are UTF-8 text, as text: what a release in a processor's request holds. */
export function textFiles(files: Map<string, Uint8Array>): Record<string, string> {
  const decoder = new TextDecoder("utf-8", { fatal: true });
  const text: Record<string, string> = {};
  for (const [path, bytes] of files) {
    try {
      text[path] = decoder.decode(bytes);
    } catch {
      // Not text: an image or a font, which the processor never reads.
    }
  }
  return text;
}
