// Asking the reader before data from a new origin (spec §8.6): a processor that fetches while a reader views a
// document asks the first time a document needs an origin the reader hasn't decided on, and remembers the answer.
// Origins the host configured are never asked about. The question comes from the host's page, never from a frame,
// whose code could draw a question of its own. Until the reader allows an origin, it isn't in `allowedOrigins`: the
// processor reports F028 and loads the import's `fallback`.

import type { Fetched } from "@compmd/processor";

/** What the reader said about an origin. */
export type OriginAnswer = "allow" | "deny";

/** What a page needs of an origin: its data, its images, or both; one answer covers both (spec §15). */
export type OriginKind = "data" | "images";

/** Asks the reader whether `document` may load what it needs, `kinds`, from `origin`. */
export type AskOrigin = (origin: string, document: string, kinds: readonly OriginKind[]) => Promise<OriginAnswer>;

/** Where the reader's answers are kept, by origin; a host may keep them elsewhere, such as on its server. */
export interface OriginAnswers {
  get(origin: string): OriginAnswer | undefined;
  set(origin: string, answer: OriginAnswer): void;
  /** Every answer kept, for a page that lets the reader change one. */
  list(): [string, OriginAnswer][];
  /** Forgets the answer about `origin`, so the reader is asked again. */
  forget(origin: string): void;
}

/** The key the default store keeps its answers under. */
const KEY = "compmd-origins";

/**
 * The reader's answers in `storage` (by default `localStorage`), for every document of the host's origin. A storage
 * that refuses to keep them, in a private window, keeps them for the page's life instead.
 *
 * @example
 * const answers = localAnswers();
 * answers.list(); // [["https://api.example.com", "allow"]]
 * answers.forget("https://api.example.com");
 */
export function localAnswers(storage: Storage | undefined = pageStorage()): OriginAnswers {
  let kept: Record<string, OriginAnswer> = {};
  try {
    kept = JSON.parse(storage?.getItem(KEY) ?? "{}") as Record<string, OriginAnswer>;
  } catch {
    kept = {};
  }
  const save = () => {
    try {
      storage?.setItem(KEY, JSON.stringify(kept));
    } catch {
      // Kept in memory only.
    }
  };
  return {
    get: (origin) => kept[origin],
    set: (origin, answer) => {
      kept[origin] = answer;
      save();
    },
    list: () => Object.entries(kept),
    forget: (origin) => {
      kept = Object.fromEntries(Object.entries(kept).filter(([known]) => known !== origin));
      save();
    },
  };
}

/** The page's `localStorage`; none where reading it throws, as in a page with an opaque origin. */
function pageStorage(): Storage | undefined {
  try {
    return globalThis.localStorage;
  } catch {
    return undefined;
  }
}

/**
 * The question's words for `origin`, by what the page needs of it, `kinds`: what the reader is asked, and what the
 * origin learns.
 *
 * @example
 * originQuestion("https://photos.example.com", ["images"]).question; // "This document wants to show images from photos.example.com. Allow?"
 */
export function originQuestion(origin: string, kinds: readonly OriginKind[] = ["data"]): { question: string; detail: string } {
  const data = kinds.includes("data");
  const images = kinds.includes("images");
  const what = data && images ? "load data and show images" : images ? "show images" : "load data";
  const learns = data ? "that this page was opened, and the values you picked on it, since they are part of what it is asked for" : "that this page was opened, and when";
  return {
    question: `This document wants to ${what} from ${new URL(origin).host}. Allow?`,
    detail: `That site learns ${learns}.`,
  };
}

/**
 * Asks with a plain `<dialog>` in the host's page, with the buttons Allow and Don't allow; closing it any other way is
 * Don't allow.
 */
export const askWithDialog: AskOrigin = (origin, _document, kinds) =>
  new Promise((resolve) => {
    const { question, detail } = originQuestion(origin, kinds);
    const dialog = document.createElement("dialog");
    dialog.className = "compmd-origin-question";
    const text = dialog.appendChild(document.createElement("p"));
    text.textContent = question;
    const more = dialog.appendChild(document.createElement("p"));
    more.textContent = detail;
    const form = dialog.appendChild(document.createElement("form"));
    form.method = "dialog";
    for (const [value, label] of [
      ["deny", "Don’t allow"],
      ["allow", "Allow"],
    ] as const) {
      const button = form.appendChild(document.createElement("button"));
      button.value = value;
      button.textContent = label;
    }
    dialog.addEventListener("close", () => {
      dialog.remove();
      resolve(dialog.returnValue === "allow" ? "allow" : "deny");
    });
    document.body.append(dialog);
    dialog.showModal();
  });

/**
 * The origins a host configured, which need no question: a list of them, or `"any"`, every origin, for a host's own
 * documents, such as a static site's or a blog's (spec §15).
 */
export type ConfiguredOrigins = readonly string[] | "any";

export interface AllowOriginsOptions {
  /** The origins the host configured, which need no question. */
  configured: ConfiguredOrigins;
  /** What the question calls the document: its title or path. */
  document: string;
  /** How the reader is asked; a plain dialog by default. */
  ask?: AskOrigin;
  /** Where the answers are kept; `localStorage` by default. */
  answers?: OriginAnswers;
}

/** The questions open now, by origin, so a reader is asked about an origin once however many renders need it. */
const open = new Map<string, Promise<OriginAnswer>>();

/**
 * The reader's answer about `origin`: the one kept in `answers`, or theirs to `ask` now, about what the page needs of it
 * (`kinds`), kept once given. A question already open for the origin, for data or images, is the one answered.
 */
export async function answerFor(origin: string, kinds: readonly OriginKind[], document: string, answers: OriginAnswers, ask: AskOrigin): Promise<OriginAnswer> {
  const kept = answers.get(origin);
  if (kept !== undefined) return kept;
  let question = open.get(origin);
  if (!question) {
    question = ask(origin, document, kinds).finally(() => open.delete(origin));
    open.set(origin, question);
  }
  const answer = await question;
  answers.set(origin, answer);
  return answer;
}

/**
 * The origins a render may fetch from: those the host configured, and those the reader allowed, asking about each
 * origin of `urls` (the remote URLs the processor's `sources` lists, `remote` and `disallowed`) the reader hasn't
 * decided on. For a host that fetches on its own; `loadData` also fetches. A host renders at once without them, then
 * again with what this gives: until then, the new origins are F028 and show the `fallback`. With `configured` `"any"`,
 * `"any"`, and nothing is asked.
 *
 * @example
 * const { remote, disallowed } = sources(request);
 * const allowedOrigins = await allowOrigins([...remote, ...disallowed], { configured: config.allowedOrigins, document: request.path });
 */
export async function allowOrigins(urls: readonly string[], options: AllowOriginsOptions): Promise<string[] | "any"> {
  if (options.configured === "any") return "any";
  const answers = options.answers ?? localAnswers();
  const ask = options.ask ?? askWithDialog;
  const allowed = new Set(options.configured);
  const needed = [...new Set(urls.map(originOf).filter((origin): origin is string => origin !== undefined))];
  for (const origin of needed) {
    if (allowed.has(origin)) continue;
    if ((await answerFor(origin, ["data"], options.document, answers, ask)) === "allow") allowed.add(origin);
  }
  return [...allowed];
}

/** What `loadData` gives a request: the origins it may fetch from, and what was fetched, by URL. */
export interface LoadedData {
  allowedOrigins: string[] | "any";
  remote: Record<string, Fetched>;
}

export interface LoadDataOptions extends AllowOriginsOptions {
  /** How a URL is fetched; the page's `fetch` by default, always without credentials or a `Referer`. */
  fetch?: typeof globalThis.fetch;
}

/**
 * A render's remote data, asking first: the origins of `urls` (what the processor's `sources` lists as `remote`
 * and `disallowed`) that the host configured or the reader allows, each asked about once (`allowOrigins`), and each
 * URL of those origins fetched without credentials or a `Referer`. A URL that can't be fetched is left out, so the
 * processor loads its `fallback`, or reports F026 without one (spec §8.6); one of an origin the reader denied is
 * F028, with its `fallback`. With no reader to ask, give an `ask` that denies: nothing is fetched then but from the
 * configured origins.
 *
 * Origins are decided when a document is validated, so a host processes the document again with what this gives.
 *
 * @example
 * const { remote, disallowed } = sources(request);
 * const loaded = await loadData([...remote, ...disallowed], { configured: config.allowedOrigins, document: title });
 * const processed = process({ ...request, ...loaded });
 */
export async function loadData(urls: readonly string[], options: LoadDataOptions): Promise<LoadedData> {
  const allowedOrigins = await allowOrigins(urls, options);
  const allowed = allowedOrigins === "any" ? "any" : new Set(allowedOrigins);
  const get = options.fetch ?? globalThis.fetch.bind(globalThis);
  const fetched = await Promise.all(
    [...new Set(urls)]
      .filter((url) => {
        const origin = originOf(url);
        return origin !== undefined && (allowed === "any" || allowed.has(origin));
      })
      .map(async (url): Promise<[string, Fetched] | undefined> => {
        try {
          // No credentials and no `Referer` with a document's fetches (spec §15).
          const response = await get(url, { credentials: "omit", referrerPolicy: "no-referrer" });
          if (!response.ok) return undefined;
          return [url, { text: await response.text(), contentType: response.headers.get("content-type") }];
        } catch {
          return undefined;
        }
      }),
  );
  return { allowedOrigins, remote: Object.fromEntries(fetched.filter((entry) => entry !== undefined)) };
}

/** The origin of `url`, as `allowedOrigins` writes it; `undefined` for text that isn't a URL. */
export function originOf(url: string): string | undefined {
  try {
    return new URL(url).origin;
  } catch {
    return undefined;
  }
}
