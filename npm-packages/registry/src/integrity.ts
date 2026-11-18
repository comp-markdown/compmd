// The integrity of a tarball, as npm publishes it (`dist.integrity`): `sha512-` and the SHA-512 of its bytes in base64
// (docs/resolving-registry-packs.md, decision 5).

/** The `sha512-…` integrity of `bytes`. */
export async function integrityOf(bytes: Uint8Array<ArrayBuffer>): Promise<string> {
  const digest = new Uint8Array(await crypto.subtle.digest("SHA-512", bytes));
  return `sha512-${btoa(String.fromCharCode(...digest))}`;
}

/** Whether `integrity` is one this package checks: a SHA-512, the only algorithm npm publishes for current packages. */
export const isChecked = (integrity: string | undefined): integrity is string => integrity?.startsWith("sha512-") === true;
