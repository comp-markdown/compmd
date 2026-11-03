// Asking the reader before data from a new origin (spec §8.6): a processor that fetches while a reader views a
// document asks the first time a document needs an origin the reader hasn't decided on, and remembers the answer.
// Origins the host configured are never asked about. The question comes from the host's page, never from a frame,
// whose code could draw a question of its own. Until the reader allows an origin, it isn't in `allowedOrigins`: the
// processor reports F028 and loads the import's `fallback`.

/** What the reader said about an origin. */
export type OriginAnswer = "allow" | "deny";

/** Asks the reader whether `document` may load data from `origin`. */
export type AskOrigin = (origin: string, document: string) => Promise<OriginAnswer>;

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

/** The question's words for `origin`: what the reader is asked, and what the origin learns. */
export function originQuestion(origin: string): { question: string; detail: string } {
  return {
    question: `This document wants to load data from ${new URL(origin).host}. Allow?`,
    detail: "That site learns the values you picked on this page, since they are part of what it is asked for.",
  };
}

/**
 * Asks with a plain `<dialog>` in the host's page, with the buttons Allow and Don't allow; closing it any other way is
 * Don't allow.
 */
export const askWithDialog: AskOrigin = (origin) =>
  new Promise((resolve) => {
    const { question, detail } = originQuestion(origin);
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

export interface AllowOriginsOptions {
  /** The origins the host configured, which need no question. */
  configured: readonly string[];
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
 * The origins a render may fetch from: those the host configured, and those the reader allowed, asking about each
 * origin of `urls` (the remote URLs the processor's `sources` lists) the reader hasn't decided on. A host renders at
 * once without them, then again with what this gives: until then, the new origins are F028 and show the `fallback`.
 *
 * @example
 * const { remote } = sources(request);
 * render(process(request)); // new origins show their fallbacks meanwhile
 * const allowedOrigins = await allowOrigins(remote, { configured: config.allowedOrigins, document: request.path });
 * render(process({ ...request, allowedOrigins, remote: await fetchFrom(remote, allowedOrigins) }));
 */
export async function allowOrigins(urls: readonly string[], options: AllowOriginsOptions): Promise<string[]> {
  const answers = options.answers ?? localAnswers();
  const ask = options.ask ?? askWithDialog;
  const allowed = new Set(options.configured);
  const needed = [...new Set(urls.map(originOf).filter((origin): origin is string => origin !== undefined))];
  for (const origin of needed) {
    if (allowed.has(origin)) continue;
    let answer = answers.get(origin);
    if (answer === undefined) {
      let question = open.get(origin);
      if (!question) {
        question = ask(origin, options.document).finally(() => open.delete(origin));
        open.set(origin, question);
      }
      answer = await question;
      answers.set(origin, answer);
    }
    if (answer === "allow") allowed.add(origin);
  }
  return [...allowed];
}

/** The origin of `url`, as `allowedOrigins` writes it; `undefined` for text that isn't a URL. */
function originOf(url: string): string | undefined {
  try {
    return new URL(url).origin;
  } catch {
    return undefined;
  }
}
