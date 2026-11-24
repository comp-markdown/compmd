// Reading a stream with a cap (audit S11): a tarball's download and its gunzip are counted as they are read, and
// stopped once they pass the host's size cap, so a pack is never held whole before it is found too large.

/**
 * The bytes of `stream`, read until its end.
 *
 * @param cap The most bytes read; one more and reading stops.
 * @param tooLarge The error's message once the stream passes `cap`.
 * @throws Error with `tooLarge` once the stream passes `cap`; the stream is cancelled.
 */
export async function readCapped(stream: ReadableStream<Uint8Array>, cap: number, tooLarge: string): Promise<Uint8Array<ArrayBuffer>> {
  const reader = stream.getReader();
  const chunks: Uint8Array[] = [];
  let length = 0;
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    length += value.length;
    if (length > cap) {
      await reader.cancel();
      throw new Error(tooLarge);
    }
    chunks.push(value);
  }
  const bytes = new Uint8Array(length);
  let offset = 0;
  for (const chunk of chunks) {
    bytes.set(chunk, offset);
    offset += chunk.length;
  }
  return bytes;
}
