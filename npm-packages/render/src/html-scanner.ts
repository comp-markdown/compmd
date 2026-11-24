// Raw HTML's tags and comments found in linear time (audit S14). A tag is `<`, an optional `/`, a name of ASCII letters,
// digits and `-` starting with a letter, attributes each after white space (a name, then optionally `=` and a value in
// double quotes, single quotes or none), white space, an optional `/` and `>`; a comment is `<!--` to the next `-->`.
// What starts with `<` and isn't one is text, and the search goes on from the next character, as a regular expression
// searched with `matchAll` would. A regular expression takes quadratic time on a long run that never closes
// (`<details <a <a …` with no `>`, or `<!--` with no `-->`), since each `<` reads the run again; here, how the
// attributes read from a place is worked out once for every tag that reaches it, and a missing `-->` is looked for once.

/** An attribute as written: its name in lower case, and its value, empty when it has none. */
export interface ScannedAttribute {
  name: string;
  value: string;
}

/** What raw HTML is made of: text, a tag (its name in lower case), or a comment, which renders as nothing. */
export type Scanned =
  | { kind: "text"; value: string }
  | { kind: "tag"; closing: boolean; tag: string; attributes: readonly ScannedAttribute[]; source: string }
  | { kind: "comment" };

/** The attributes read from a place to the tag's end, sharing their tail with every tag that reaches the same place. */
interface Section {
  attributes: Attributes | undefined;
  /** Just after the tag's `>`. */
  end: number;
}

/** Attributes as a list each tag reaching a place shares. */
interface Attributes {
  first: ScannedAttribute;
  rest: Attributes | undefined;
}

const isSpace = (char: string | undefined): boolean => char !== undefined && /\s/.test(char);
const isLetter = (char: string | undefined): boolean => char !== undefined && /[a-zA-Z]/.test(char);
const isTagChar = (char: string | undefined): boolean => char !== undefined && /[a-zA-Z0-9-]/.test(char);
const isNameChar = (char: string | undefined): boolean => char !== undefined && !isSpace(char) && !`"'>/=`.includes(char);
const isValueChar = (char: string | undefined): boolean => char !== undefined && !isSpace(char) && !"\"'=<>`".includes(char);

/** One step of an attribute section from a place: an attribute and where the section goes on, its end, or a failure. */
type Step = { attribute: ScannedAttribute; next: number } | { end: number } | undefined;

/** Reads raw HTML, one string. */
class Scanner {
  readonly #html: string;
  /** How the attributes read from each place reached so far; `null` where no tag can end. */
  readonly #sections = new Map<number, Section | null>();
  /** Where no `-->` follows, once a search found none. */
  #noCommentEndFrom = Number.POSITIVE_INFINITY;

  constructor(html: string) {
    this.#html = html;
  }

  scan(): Scanned[] {
    const html = this.#html;
    const scanned: Scanned[] = [];
    let text = 0;
    let at = html.indexOf("<");
    while (at !== -1) {
      const found = this.#commentAt(at) ?? this.#tagAt(at);
      if (found === undefined) {
        at = html.indexOf("<", at + 1);
        continue;
      }
      if (at > text) scanned.push({ kind: "text", value: html.slice(text, at) });
      scanned.push(found.scanned);
      text = found.end;
      at = html.indexOf("<", found.end);
    }
    if (text < html.length) scanned.push({ kind: "text", value: html.slice(text) });
    return scanned;
  }

  /** A comment at `at`, to the first `-->` after its `<!--`. */
  #commentAt(at: number): { scanned: Scanned; end: number } | undefined {
    if (!this.#html.startsWith("<!--", at)) return undefined;
    const from = at + 4;
    if (from >= this.#noCommentEndFrom) return undefined;
    const close = this.#html.indexOf("-->", from);
    if (close === -1) {
      this.#noCommentEndFrom = from;
      return undefined;
    }
    return { scanned: { kind: "comment" }, end: close + 3 };
  }

  /** A tag at `at`. */
  #tagAt(at: number): { scanned: Scanned; end: number } | undefined {
    const html = this.#html;
    const closing = html[at + 1] === "/";
    const nameStart = at + (closing ? 2 : 1);
    if (!isLetter(html[nameStart])) return undefined;
    let nameEnd = nameStart + 1;
    while (isTagChar(html[nameEnd])) nameEnd += 1;
    const section = this.#section(nameEnd);
    if (section === null) return undefined;
    const attributes: ScannedAttribute[] = [];
    for (let list = section.attributes; list; list = list.rest) attributes.push(list.first);
    const tag = html.slice(nameStart, nameEnd).toLowerCase();
    return { scanned: { kind: "tag", closing, tag, attributes, source: html.slice(at, section.end) }, end: section.end };
  }

  /** The attributes read from `from` to the tag's end, worked out once for each place; `null` when no tag ends. */
  #section(from: number): Section | null {
    const walked: { place: number; attribute: ScannedAttribute }[] = [];
    let place = from;
    let result: Section | null;
    for (;;) {
      const known = this.#sections.get(place);
      if (known !== undefined) {
        result = known;
        break;
      }
      const step = this.#step(place);
      if (step === undefined || "end" in step) {
        result = step === undefined ? null : { attributes: undefined, end: step.end };
        this.#sections.set(place, result);
        break;
      }
      walked.push({ place, attribute: step.attribute });
      place = step.next;
    }
    for (const { place: walkedPlace, attribute } of walked.reverse()) {
      result = result === null ? null : { attributes: { first: attribute, rest: result.attributes }, end: result.end };
      this.#sections.set(walkedPlace, result);
    }
    return result;
  }

  /** What follows `at` in an attribute section: white space and an attribute, or the tag's end. */
  #step(at: number): Step {
    const html = this.#html;
    let next = at;
    while (isSpace(html[next])) next += 1;
    if (next > at && isNameChar(html[next])) {
      const nameStart = next;
      while (isNameChar(html[next])) next += 1;
      const name = html.slice(nameStart, next).toLowerCase();
      let equals = next;
      while (isSpace(html[equals])) equals += 1;
      if (html[equals] !== "=") return { attribute: { name, value: "" }, next };
      let value = equals + 1;
      while (isSpace(html[value])) value += 1;
      const quote = html[value];
      if (quote === '"' || quote === "'") {
        const close = html.indexOf(quote, value + 1);
        return close === -1 ? undefined : { attribute: { name, value: html.slice(value + 1, close) }, next: close + 1 };
      }
      if (!isValueChar(quote)) return undefined;
      let end = value;
      while (isValueChar(html[end])) end += 1;
      return { attribute: { name, value: html.slice(value, end) }, next: end };
    }
    if (html[next] === "/") next += 1;
    return html[next] === ">" ? { end: next + 1 } : undefined;
  }
}

/**
 * The text, tags and comments of `html`, in order, in time linear in its length.
 *
 * @example
 * scanHtml('a <kbd class="k">'); // [{ kind: "text", value: "a " }, { kind: "tag", tag: "kbd", attributes: [{ name: "class", value: "k" }], … }]
 */
export function scanHtml(html: string): Scanned[] {
  return new Scanner(html).scan();
}
