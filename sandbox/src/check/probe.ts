// The check's probe: a strict component whose frame reports what it can see and do (docs/hosting.md, decision 11).

/** What the probe's frame reports. */
export interface Seen {
  /** `self.origin`: `"null"` in a sandboxed frame. */
  origin: string;
  /** Each host API name the adapter gave, and whether the frame sees it. */
  hostApi: Record<string, boolean>;
  /** Whether WebKit's `ipc` script message handler is there, which a policy doesn't govern (the Tauri plugin's Risks). */
  webkitIpc: boolean;
  /** Whether a `fetch` of the frames' own document was refused, as a strict frame's policy refuses every request. */
  fetchRefused: boolean;
}

/**
 * The probe's code: a plain component that reports `Seen` as the JSON of its input `result`, looking for the globals
 * `hostApi` names and fetching `url`.
 */
export function probeCode(hostApi: readonly string[]): string {
  const names = JSON.stringify(hostApi);
  return `export default function mount(target, context) {
  const url = String(context.params.url ?? "");
  const hostApi = Object.fromEntries(${names}.map((name) => [name, name in self]));
  const webkitIpc = Boolean(self.webkit && self.webkit.messageHandlers && self.webkit.messageHandlers.ipc);
  const report = (fetchRefused) => context.set("result", JSON.stringify({ origin: String(self.origin), hostApi, webkitIpc, fetchRefused }));
  fetch(url).then(() => report(false), () => report(true));
  target.textContent = "the check's probe";
}`;
}

/** What the probe reported, from its input's value; `undefined` when it isn't a report. */
export function seenOf(value: unknown): Seen | undefined {
  try {
    const seen = JSON.parse(String(value)) as Partial<Seen>;
    return typeof seen.origin === "string" && typeof seen.hostApi === "object" && typeof seen.webkitIpc === "boolean" && typeof seen.fetchRefused === "boolean" ? (seen as Seen) : undefined;
  } catch {
    return undefined;
  }
}
