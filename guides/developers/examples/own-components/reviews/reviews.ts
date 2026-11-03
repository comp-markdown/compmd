/** What a permissive component given `host` gets: the app's functions, called by name, answered by its page. */
interface Host {
  call(name: string, ...args: unknown[]): Promise<unknown>;
}

/** What the renderer gives a plain component (spec §7.4), as far as `reviews` uses it. */
interface Context {
  params: Record<string, unknown>;
  host?: unknown;
}

/** What the app answers for a book it has reviews of. */
interface Summary {
  count: number;
  average: number;
}

const isSummary = (value: unknown): value is Summary =>
  typeof value === "object" && value !== null && typeof (value as Summary).count === "number" && typeof (value as Summary).average === "number";

// A summary of a book's reviews, which it asks the app for: the reviews are the app's data, never the document's.
export default function reviews(target: Element, { params, host }: Context) {
  const line = target.ownerDocument.createElement("p");
  target.append(line);
  /** How many times it asked: an answer to an older question is dropped. */
  let asked = 0;
  const show = async (next: Record<string, unknown>) => {
    if (host === undefined) {
      line.textContent = "Reviews aren't available here.";
      return;
    }
    const question = ++asked;
    line.textContent = "Asking for the reviews…";
    let summary: unknown;
    try {
      summary = await (host as Host).call("reviewsOf", String(next.book));
    } catch {
      summary = undefined;
    }
    if (question !== asked) return;
    line.textContent = isSummary(summary) ? `${String(summary.count)} readers gave it ${summary.average.toFixed(1)} of 5 on average.` : "No reviews yet.";
  };
  void show(params);
  return {
    update: (next: Record<string, unknown>) => {
      void show(next);
    },
  };
}
