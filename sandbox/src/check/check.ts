// The hosting check (docs/hosting.md, decision 11): what every host runs in its real browser or webview, so a host
// knows it serves the bundle as the sandbox needs: a strict frame from its origin, opaque, with no host API and no
// network; a highlighted code block; packs' modules by URL; the contract's headers, and nothing outside it.

import type { HostComponent } from "@compmd/processor";
import type { ExpandedComponent, ExpandedRoot } from "@compmd/render";
import type { BundleModuleSource } from "../found.ts";
import { givenOrigin } from "../given-origin.ts";
import { createSandbox, type ViewEvents } from "../sandbox.ts";
import { CONTRACT_HEADERS } from "./contract.ts";
import { probeCode, seenOf, type Seen } from "./probe.ts";

type Place = NonNullable<HostComponent["module"]>;

/** A pack's module the check draws: its place, and the params its component needs to draw. */
export interface CheckedPack {
  place: Place;
  params?: Readonly<Record<string, unknown>>;
}

export interface CheckOptions {
  /** Where the check draws its document, in the host's page. */
  container: HTMLElement;
  /** The bundle's frames under the origin, `frames/<hash>/` (`virtual:compmd`'s `sandbox.frames`). */
  frames: string;
  /** Where the bundle is served; the adapter's `globalThis.__COMPMD__.origin` without it. */
  origin?: string;
  /** Where the packs' modules are: the bundle's (`virtual:compmd`'s `sandbox.modules`) and the host's store's. */
  modules?: BundleModuleSource;
  /** The host's API, the globals a frame must not see: `__TAURI_INTERNALS__`, Electron's `ipcRenderer`. */
  hostApi?: readonly string[];
  /** Packs' modules that must load by their URL and draw: a bundled pack's, and one from the host's store. */
  packs?: readonly CheckedPack[];
  /** How long the check waits for what it looks at, in ms (30 000 unless given). */
  timeout?: number;
}

/** What the check found: whether every check passed, each check that passed and each that failed, with why. */
export interface CheckReport {
  ok: boolean;
  passed: string[];
  failed: string[];
  /** One line for an adapter's output: what passed, or what failed. */
  message: string;
}

/** A hash no build of the frames has, for a path outside the contract. */
const NO_BUILD = "frames/0000000000000000/strict.html";

/** One check's outcome: a description, and why it failed when it did. */
type Outcome = { check: string; failure?: string };

const passing = (check: string): Outcome => ({ check });
const failing = (check: string, failure: string): Outcome => ({ check, failure });

/** Waits until `done` holds, or `ms` has passed. */
async function until(done: () => boolean, ms: number): Promise<void> {
  const end = Date.now() + ms;
  while (!done() && Date.now() < end) await new Promise((resolve) => setTimeout(resolve, 50));
}

/** What a request for `url` answers: its status and headers, or why the page couldn't ask. */
async function asked(url: string): Promise<{ status: number; headers: Headers } | { error: string }> {
  try {
    const response = await fetch(url, { cache: "no-store" });
    return { status: response.status, headers: response.headers };
  } catch (error) {
    return { error: error instanceof Error ? error.message : String(error) };
  }
}

/** Whether the file at `url` arrives with the contract's headers, exactly. */
async function servedFile(url: string): Promise<Outcome> {
  const check = `\`${url}\` is served with the contract's headers`;
  const answer = await asked(url);
  if ("error" in answer) return failing(check, `the page couldn't fetch it: ${answer.error} (does its policy allow \`connect-src\` to the frames?)`);
  if (answer.status !== 200) return failing(check, `it answered ${String(answer.status)}`);
  const wrong = Object.entries(CONTRACT_HEADERS).filter(([name, value]) => answer.headers.get(name) !== value);
  return wrong.length === 0 ? passing(check) : failing(check, wrong.map(([name, value]) => `${name} is ${JSON.stringify(answer.headers.get(name))}, not ${JSON.stringify(value)}`).join("; "));
}

/** Whether `url`, outside the contract, is refused with a `404` the page can read. */
async function refused(url: string): Promise<Outcome> {
  const check = `\`${url}\`, outside the contract, is a 404`;
  const answer = await asked(url);
  if ("error" in answer) return failing(check, `the page couldn't read the answer: ${answer.error} (a refusal needs \`Access-Control-Allow-Origin: *\`)`);
  return answer.status === 404 ? passing(check) : failing(check, `it answered ${String(answer.status)}`);
}

/** What the probe's frame reports, as checks. */
function framed(seen: Seen | undefined, hostApi: readonly string[]): Outcome[] {
  const check = (name: string, ok: (seen: Seen) => boolean, failure: (seen: Seen) => string): Outcome =>
    seen === undefined ? failing(name, "the probe didn't report: its strict frame didn't load or run") : ok(seen) ? passing(name) : failing(name, failure(seen));
  return [
    check("a strict frame's origin is opaque", ({ origin }) => origin === "null", ({ origin }) => `its origin is ${origin}`),
    check("a strict frame's `fetch` is refused", ({ fetchRefused }) => fetchRefused, () => "it fetched the frames' own document"),
    check("a strict frame has no WebKit `ipc` message handler", ({ webkitIpc }) => !webkitIpc, () => "`window.webkit.messageHandlers.ipc` is there, which no policy governs"),
    ...hostApi.map((name) => check(`a strict frame has no \`${name}\``, (seen) => seen.hostApi[name] === false, () => `\`${name}\` is there: a frame's code could reach the host`)),
  ];
}

/** A leaf of the component `name`, with `params`, its input `result` bound to `input`. */
function leaf(name: string, params: Readonly<Record<string, unknown>>, input?: string): ExpandedComponent {
  const given = Object.entries(params).map(([key, value]) => ({ key, breakpoint: null, value }));
  return { type: "componentLeaf", name, params: [...given, ...(input === undefined ? [] : [{ key: "result", breakpoint: null, value: "", input }])], fallback: [{ type: "text", value: `${name}'s fallback` }] };
}

/**
 * Runs the check in the host's page: draws its document in `container`, in host mode, and asks the host for the
 * frames' files, and reports what passed and what failed. An adapter turns the report into an exit code or a test.
 *
 * @example
 * import { sandbox } from "virtual:compmd";
 * const report = await runCheck({ container, frames: sandbox.frames, modules: sandbox.modules, hostApi: ["__TAURI_INTERNALS__"], packs: [{ place, params }] });
 * if (!report.ok) console.error(report.message);
 */
export async function runCheck(options: CheckOptions): Promise<CheckReport> {
  const origin = options.origin ?? givenOrigin();
  if (origin === undefined) return report([failing("the bundle has an origin", "neither the options nor the host's adapter (`__COMPMD__`) give one")]);
  const at = new URL(origin.replace(/\/*$/, "/"), location.href);
  const base = new URL(options.frames, at).href;
  const hostApi = options.hostApi ?? [];
  const ms = options.timeout ?? 30_000;

  const served = await Promise.all([servedFile(`${base}strict.html`), servedFile(`${base}runtime.js`)]);
  const outside = await Promise.all([refused(new URL(NO_BUILD, at).href), refused(`${base}not-a-file-of-the-frames.js`)]);

  // The packs' modules, asked of the host: each must come by its URL, not as code.
  const byUrl = new Map<string, boolean>();
  const modules: BundleModuleSource = (module) => {
    const given = options.modules?.(module);
    byUrl.set(`${module.pack}@${module.version}/${module.path}`, given !== undefined && !("code" in given));
    return given;
  };
  const packs = options.packs ?? [];
  const components: HostComponent[] = [
    { name: "probe", language: "js", code: probeCode(hostApi), trust: { level: "strict" } },
    ...packs.map(({ place }, i): HostComponent => ({ name: `pack-${String(i)}`, language: "js", code: "", trust: { level: "strict" }, module: place })),
  ];
  const tree: ExpandedRoot = {
    type: "root",
    children: [
      { type: "code", lang: "rust", meta: null, value: 'fn main() {\n    println!("the check");\n}' },
      leaf("probe", { url: `${base}strict.html` }, "seen"),
      ...packs.map(({ params }, i) => leaf(`pack-${String(i)}`, params ?? {})),
    ],
    inputs: { seen: { type: "string", value: "" } },
  };
  const view = createSandbox({ origin, frames: options.frames, modules, page: "host" }).open(options.container, { components });
  const observation = view.observe(() => undefined);
  const errors: ViewEvents["error"][] = [];
  let seenValue: unknown;
  view.on("error", (error) => errors.push(error));
  view.on("set", ({ input, value }) => {
    if (input === "seen") seenValue = value;
  });
  view.render(tree);

  const mounted = (name: string) => observation.events.some((event) => event.type === "mounted" && event.component === name && event.frame !== "");
  const highlighted = () => (options.container.querySelector("pre code.language-rust")?.querySelectorAll("span").length ?? 0) > 1;
  await until(() => (seenValue !== undefined && highlighted() && packs.every((_pack, i) => mounted(`pack-${String(i)}`))) || errors.length > 0, ms);

  const frames = [...options.container.querySelectorAll("iframe")].map((frame) => frame.src);
  const drawn: Outcome[] = [
    frames.length > 0 && frames.every((src) => src.startsWith(base)) ? passing("a strict frame loads from the origin") : failing("a strict frame loads from the origin", `the frames are ${JSON.stringify(frames)}, not under ${base}`),
    ...framed(seenOf(seenValue), hostApi),
    highlighted() ? passing("a code block is highlighted") : failing("a code block is highlighted", "the Rust block has no tokens: the page didn't load the frames' `highlight.js` (does its policy allow `script-src` to the frames?)"),
    ...packs.map(({ place }, i): Outcome => {
      const name = `\`${place.pack}@${place.version}/${place.path}\``;
      const check = `${name} loads by its URL and draws`;
      const problem = errors.find(({ component }) => component === `pack-${String(i)}`);
      if (byUrl.get(`${place.pack}@${place.version}/${place.path}`) === false) return failing(check, "the host has no URL for it");
      if (problem) return failing(check, problem.message);
      return mounted(`pack-${String(i)}`) ? passing(check) : failing(check, "it didn't mount in its frame");
    }),
  ];
  const other = errors.filter(({ component }) => !/^pack-\d+$/.test(component));
  return report([...served, ...outside, ...drawn, ...(other.length > 0 ? [failing("the check's document draws without errors", other.map(({ component, message }) => `${component}: ${message}`).join("; "))] : [])]);
}

/** The report of `outcomes`. */
function report(outcomes: readonly Outcome[]): CheckReport {
  const failed = outcomes.filter((outcome) => outcome.failure !== undefined).map(({ check, failure }) => `${check}: ${failure ?? ""}`);
  const passed = outcomes.filter((outcome) => outcome.failure === undefined).map(({ check }) => check);
  return { ok: failed.length === 0, passed, failed, message: failed.length === 0 ? `passed: ${passed.join("; ")}` : `failed: ${failed.join("; ")}` };
}
