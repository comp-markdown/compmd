// Links to a passage of a document's text as text fragments, the web's way to link to text (https://wicg.github.io/
// scroll-to-text-fragment/): `#:~:text=[prefix-,]start[,end][,-suffix]`, each part percent-encoded. A page shows the
// passage in its rendered text; the editor finds it in the document's source.

/** A text directive: the passage starts with `start` and, when given, ends with `end`, after `prefix` and before `suffix`. */
export interface TextDirective {
  prefix?: string;
  start: string;
  end?: string;
  suffix?: string;
}

/** A passage of a text: `start` to `end`, in UTF-16 units. */
export interface Passage {
  start: number;
  end: number;
}

/** What a text fragment's directive says, from a link (`#:~:text=…`, `:~:text=…`); `undefined` for any other link. */
export function parseFragment(link: string): TextDirective | undefined {
  const at = link.indexOf(":~:text=");
  if (at === -1) return undefined;
  // Only the first directive: `&` starts another.
  const parts = (link.slice(at + ":~:text=".length).split("&")[0] ?? "").split(",");
  const decoded = (part: string) => {
    try {
      return decodeURIComponent(part);
    } catch {
      return undefined;
    }
  };
  const first = parts[0];
  const prefix = first?.endsWith("-") ? decoded(first.slice(0, -1)) : undefined;
  const rest = prefix === undefined ? parts : parts.slice(1);
  const last = rest.at(-1);
  const suffix = rest.length > 1 && last?.startsWith("-") ? decoded(last.slice(1)) : undefined;
  const middle = suffix === undefined ? rest : rest.slice(0, -1);
  if (middle.length < 1 || middle.length > 2) return undefined;
  const [start, end] = middle.map(decoded);
  if (!start || (middle.length === 2 && !end)) return undefined;
  return { ...(prefix ? { prefix } : {}), start, ...(end ? { end } : {}), ...(suffix ? { suffix } : {}) };
}

/** The first passage of `text` that `directive` names: its prefix and suffix, when given, next to it but for white space. */
export function findPassage(text: string, directive: TextDirective): Passage | undefined {
  for (let at = text.indexOf(directive.start); at !== -1; at = text.indexOf(directive.start, at + 1)) {
    if (directive.prefix !== undefined && !text.slice(0, at).trimEnd().endsWith(directive.prefix)) continue;
    let end = at + directive.start.length;
    if (directive.end !== undefined) {
      const last = text.indexOf(directive.end, end);
      if (last === -1) return undefined;
      end = last + directive.end.length;
    }
    if (directive.suffix !== undefined && !text.slice(end).trimStart().startsWith(directive.suffix)) continue;
    return { start: at, end };
  }
  return undefined;
}

/** A part of a directive, percent-encoded: `-`, `,` and `&` too, which the syntax uses. */
const encoded = (part: string) => encodeURIComponent(part).replaceAll("-", "%2D");

/** The fragment of `directive`, `:~:text=…`. */
export function fragmentOf({ prefix, start, end, suffix }: TextDirective): string {
  const parts = [prefix === undefined ? undefined : `${encoded(prefix)}-`, encoded(start), end === undefined ? undefined : encoded(end), suffix === undefined ? undefined : `-${encoded(suffix)}`];
  return `:~:text=${parts.filter((part) => part !== undefined).join(",")}`;
}

/** Up to `count` words of `text`, from its end (`fromEnd`) or its start, on the nearest line that has any. */
function words(text: string, count: number, fromEnd: boolean): string | undefined {
  const line = fromEnd ? (text.trimEnd().split("\n").at(-1) ?? "") : (text.trimStart().split("\n")[0] ?? "");
  const all = line.trim().split(/\s+/).filter(Boolean);
  const taken = fromEnd ? all.slice(-count) : all.slice(0, count);
  return taken.length > 0 ? taken.join(" ") : undefined;
}

/** A passage this long or longer, or on several lines, is named by its start and end. */
const LONG = 80;

/**
 * The directive that names `passage` of `text`: the passage itself when it is short and on one line, else its first and
 * last words; with the words before or after it when the text has the same words earlier. `undefined` for an empty
 * passage.
 *
 * @example
 * directiveFor("a: 1\nb: 1\n", { start: 8, end: 9 }); // { prefix: "b:", start: "1" }
 */
export function directiveFor(text: string, passage: Passage): TextDirective | undefined {
  const selected = text.slice(passage.start, passage.end);
  if (selected.trim() === "") return undefined;
  const long = selected.includes("\n") || selected.length >= LONG;
  const base: TextDirective = long ? { start: words(selected, 4, false) ?? selected, end: words(selected, 4, true) ?? selected } : { start: selected };
  const before = words(text.slice(0, passage.start), 3, true);
  const after = words(text.slice(passage.end), 3, false);
  const candidates: TextDirective[] = [
    base,
    ...(before === undefined ? [] : [{ ...base, prefix: before }]),
    ...(after === undefined ? [] : [{ ...base, suffix: after }]),
    ...(before === undefined || after === undefined ? [] : [{ ...base, prefix: before, suffix: after }]),
  ];
  const names = (candidate: TextDirective) => {
    const found = findPassage(text, candidate);
    return found?.start === passage.start && found.end === passage.end;
  };
  return candidates.find(names) ?? base;
}
