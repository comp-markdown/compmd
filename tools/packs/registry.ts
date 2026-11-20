// What the npm registry has of a package, asked over HTTP: npm itself doesn't run inside this repository, whose
// `devEngines` names pnpm (RELEASING.md, *What publishing 0.1.0 taught*).

export const REGISTRY = "https://registry.npmjs.org";

/** What `versionsOf` needs of `fetch`. */
export type Fetch = (url: string) => Promise<{ status: number; ok: boolean; json(): Promise<unknown> }>;

/** The registry's address of `name`, its scope's slash escaped: `https://registry.npmjs.org/@comp-markdown%2fcard`. */
export const packageUrl = (name: string): string => `${REGISTRY}/${name.replace("/", "%2f")}`;

/**
 * The versions the registry has of `name`; empty when it has no such package.
 *
 * @throws Error when the registry answers anything but the package or "not found".
 * @example
 * await versionsOf("@comp-markdown/mermaid"); // ["0.0.0-stage", "0.1.0", …]
 */
export async function versionsOf(name: string, fetch: Fetch = globalThis.fetch): Promise<string[]> {
  const response = await fetch(packageUrl(name));
  if (response.status === 404) return [];
  if (!response.ok) throw new Error(`the registry answered ${String(response.status)} for ${name}`);
  const { versions = {} } = (await response.json()) as { versions?: Record<string, unknown> };
  return Object.keys(versions);
}
