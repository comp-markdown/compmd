// Tar archives written for the tests, entry by entry, as the reader's cases need them: a long name, a link, a size
// that isn't a whole number.

/** A tar header for `name`, `size` bytes (or the size field as written) of type `type`, with `prefix` in the ustar prefix field. */
export function header(name: string, size: number | string, type: string, prefix = ""): Uint8Array {
  const block = new Uint8Array(512);
  const put = (text: string, at: number) => {
    block.set(new TextEncoder().encode(text), at);
  };
  put(name, 0);
  put(typeof size === "string" ? size : size.toString(8).padStart(11, "0"), 124);
  put(type, 156);
  put("ustar", 257);
  put(prefix, 345);
  return block;
}

/** An archive of `entries`, each a header and its data, padded to blocks, then the two empty blocks that end it. */
export function archive(...entries: [name: string, data: string | Uint8Array, type?: string, prefix?: string][]): Uint8Array<ArrayBuffer> {
  const parts = entries.flatMap(([name, data, type = "0", prefix]) => {
    const bytes = typeof data === "string" ? new TextEncoder().encode(data) : data;
    const padded = new Uint8Array(Math.ceil(bytes.length / 512) * 512);
    padded.set(bytes);
    return [header(name, bytes.length, type, prefix), padded];
  });
  const all = [...parts, new Uint8Array(1024)];
  const out = new Uint8Array(all.reduce((length, part) => length + part.length, 0));
  let at = 0;
  for (const part of all) {
    out.set(part, at);
    at += part.length;
  }
  return out;
}
