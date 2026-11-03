// Search: the Search button in the site's header, or the `/` key, opens a dialog that searches every section of the
// site, the spec and the guides as you type. The index (`build/search.ts`) and MiniSearch load the first time it
// opens, so a page that is only read loads neither. A result followed opens at the first of its words in its
// section, all of them marked for a few seconds.
import type MiniSearch from "minisearch";
import type { Section } from "../build/search.ts";

/** The most results shown. */
const SHOWN = 20;

/** A part of a snippet, marked when it is one of the words searched for. */
export interface Part {
  text: string;
  marked: boolean;
}

const escaped = (word: string) => word.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

/**
 * Up to `width` characters of `text` around the first of `terms` in it, with each of `terms` marked; the start of
 * `text` when none is in it.
 *
 * @example
 * snippet("Data from a website, fetched while it is read.", ["fetch"], 20);
 * // [{ text: "…website, ", marked: false }, { text: "fetch", marked: true }, { text: "ed while…", marked: false }]
 */
export function snippet(text: string, terms: readonly string[], width = 160): Part[] {
  const words = terms.filter(Boolean).map(escaped);
  const matcher = words.length > 0 ? new RegExp(`(${words.join("|")})`, "gi") : undefined;
  const first = matcher ? text.search(matcher) : -1;
  // Cut at spaces, so no word shows in part.
  let start = first <= width / 3 ? 0 : first - Math.floor(width / 3);
  const after = text.indexOf(" ", start);
  if (start > 0 && after !== -1 && after < first) start = after + 1;
  let end = Math.min(text.length, start + width);
  const before = text.lastIndexOf(" ", end);
  if (end < text.length && before > Math.max(first, start)) end = before;
  const shown = `${start > 0 ? "…" : ""}${text.slice(start, end)}${end < text.length ? "…" : ""}`;
  if (!matcher) return [{ text: shown, marked: false }];
  return shown
    .split(matcher)
    .filter(Boolean)
    .map((part) => ({ text: part, marked: words.some((word) => new RegExp(`^${word}$`, "i").test(part)) }));
}

/** The index, searched as a reader types: titles count more, and words match by their start and with a typo. */
async function loadIndex(root: string): Promise<MiniSearch<Section>> {
  const [{ default: Search }, sections] = await Promise.all([
    import("minisearch"),
    fetch(`${root}search.json`).then((response) => response.json() as Promise<Section[]>),
  ]);
  const index = new Search<Section>({
    fields: ["title", "text"],
    storeFields: ["url", "page", "title", "text"],
    searchOptions: { prefix: true, fuzzy: 0.2, boost: { title: 3 } },
  });
  index.addAll(sections);
  return index;
}

/** Where a result followed is kept until its page has loaded: its address and the words found. */
const FOLLOWED = "compmd-search-followed";
/** How long the words found stay marked, in ms. */
const MARKED_FOR = 4000;

/**
 * The words `terms` in the section the heading `id` starts, to the next heading of its level or above: the first is
 * scrolled to, and all are marked for a few seconds, without changing the page (CSS Custom Highlight API). Where the
 * browser has no highlights, the section's heading is where the page already is.
 */
export function markSection(document: Document, id: string, terms: readonly string[]): void {
  const heading = document.getElementById(id);
  const words = terms.filter(Boolean).map(escaped);
  if (!heading || words.length === 0 || typeof Highlight === "undefined") return;
  const level = Number(heading.tagName.slice(1)) || 6;
  const ends = (element: Element) => /^H[1-6]$/.test(element.tagName) && Number(element.tagName.slice(1)) <= level;
  const matcher = new RegExp(words.join("|"), "gi");
  const ranges: Range[] = [];
  const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
  walker.currentNode = heading;
  for (let node = walker.nextNode(); node; node = walker.nextNode()) {
    const block = node.parentElement?.closest("h1, h2, h3, h4, h5, h6");
    if (block && block !== heading && ends(block)) break;
    if (node.parentElement?.closest(".anchor, .source, dialog")) continue;
    for (const match of (node.textContent ?? "").matchAll(matcher)) {
      const range = document.createRange();
      range.setStart(node, match.index);
      range.setEnd(node, match.index + match[0].length);
      ranges.push(range);
    }
  }
  const [first] = ranges;
  if (!first) return;
  CSS.highlights.set("compmd-search", new Highlight(...ranges));
  first.startContainer.parentElement?.scrollIntoView({ block: "center" });
  setTimeout(() => CSS.highlights.delete("compmd-search"), MARKED_FOR);
}

/** The result followed to this page, if it is this page: its words marked once the page is there. */
function markFollowed(document: Document): void {
  let followed: { href: string; terms: string[] } | undefined;
  try {
    const kept = sessionStorage.getItem(FOLLOWED);
    sessionStorage.removeItem(FOLLOWED);
    followed = kept ? (JSON.parse(kept) as { href: string; terms: string[] }) : undefined;
  } catch {
    return;
  }
  const url = followed && new URL(followed.href, location.href);
  if (!followed || !url || url.pathname !== location.pathname || !url.hash) return;
  markSection(document, decodeURIComponent(url.hash.slice(1)), followed.terms);
}

/** Whether a key press is typing, in a field, so `/` is a character there. */
const typing = (target: EventTarget | null) =>
  target instanceof HTMLElement && (target.isContentEditable || ["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName));

/** The search dialog, made the first time it opens. */
function searchDialog(document: Document, root: string): HTMLDialogElement {
  const dialog = document.createElement("dialog");
  dialog.className = "search";
  dialog.setAttribute("aria-label", "Search the site");
  const input = dialog.appendChild(document.createElement("input"));
  input.type = "search";
  input.placeholder = "Search the spec and the guides";
  input.setAttribute("aria-label", "Search the site");
  input.autocomplete = "off";
  const status = dialog.appendChild(document.createElement("p"));
  status.className = "search-status";
  status.setAttribute("role", "status");
  const list = dialog.appendChild(document.createElement("ol"));
  list.className = "search-results";
  let index: MiniSearch<Section> | undefined;
  void loadIndex(root).then(
    (loaded) => {
      index = loaded;
      show();
    },
    () => {
      status.textContent = "The search index couldn't be loaded. Reload the page to try again.";
    },
  );
  const links = () => [...list.querySelectorAll("a")];
  function show(): void {
    const query = input.value.trim();
    list.replaceChildren();
    if (!index) {
      status.textContent = query ? "Loading the index…" : "";
      return;
    }
    if (!query) {
      status.textContent = "";
      return;
    }
    const results = index.search(query).slice(0, SHOWN);
    status.textContent = results.length === 0 ? `No section matches “${query}”.` : "";
    for (const result of results) {
      const section = result as unknown as Section & { terms: string[] };
      const link = list.appendChild(document.createElement("li")).appendChild(document.createElement("a"));
      link.href = `${root}${section.url}`;
      link.dataset.terms = JSON.stringify(result.terms);
      const title = link.appendChild(document.createElement("span"));
      title.className = "search-title";
      title.textContent = section.title;
      const where = link.appendChild(document.createElement("span"));
      where.className = "search-where";
      where.textContent = section.page;
      const text = link.appendChild(document.createElement("span"));
      text.className = "search-snippet";
      for (const part of snippet(section.text, result.terms)) {
        text.append(part.marked ? Object.assign(document.createElement("mark"), { textContent: part.text }) : part.text);
      }
    }
  }
  input.addEventListener("input", show);
  input.addEventListener("keydown", (event) => {
    if (event.key === "Enter") {
      event.preventDefault();
      links()[0]?.click();
    } else if (event.key === "ArrowDown") {
      event.preventDefault();
      links()[0]?.focus();
    }
  });
  list.addEventListener("keydown", (event) => {
    const all = links();
    const at = all.indexOf(event.target as HTMLAnchorElement);
    if (at === -1 || (event.key !== "ArrowDown" && event.key !== "ArrowUp")) return;
    event.preventDefault();
    const next = event.key === "ArrowDown" ? all[at + 1] : at === 0 ? input : all[at - 1];
    next?.focus();
  });
  // A result followed: its words are marked where it opens, on this page once it has scrolled there, on another once
  // that page has loaded. The dialog closes for a result on this page.
  list.addEventListener("click", (event) => {
    const link = event.target instanceof Element ? event.target.closest("a") : null;
    if (!link) return;
    const terms = JSON.parse(link.dataset.terms ?? "[]") as string[];
    dialog.close();
    if (link.pathname === location.pathname) {
      setTimeout(() => {
        markSection(document, decodeURIComponent(link.hash.slice(1)), terms);
      }, 0);
      return;
    }
    try {
      sessionStorage.setItem(FOLLOWED, JSON.stringify({ href: link.href, terms }));
    } catch {
      // Without storage, the page opens at the section, unmarked.
    }
  });
  // A click on the backdrop, outside the dialog's box, closes it.
  dialog.addEventListener("click", (event) => {
    if (event.target === dialog) dialog.close();
  });
  document.body.append(dialog);
  return dialog;
}

/**
 * Wires the Search button of `document`'s header, and the `/` key, to the search dialog.
 *
 * @example
 * attachSearch(document);
 */
export function attachSearch(document: Document): void {
  markFollowed(document);
  const button = document.querySelector<HTMLButtonElement>("header.site button.search");
  if (!button) return;
  const root = button.dataset.root ?? "";
  let dialog: HTMLDialogElement | undefined;
  const open = () => {
    dialog ??= searchDialog(document, root);
    if (!dialog.open) dialog.showModal();
    dialog.querySelector("input")?.select();
  };
  button.addEventListener("click", open);
  document.addEventListener("keydown", (event) => {
    if (event.key !== "/" || event.metaKey || event.ctrlKey || event.altKey || typing(event.target)) return;
    event.preventDefault();
    open();
  });
}
