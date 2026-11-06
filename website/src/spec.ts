// The spec page's script, on top of its static HTML: the contents mark the section being read, and fold into a menu on
// a narrow screen, and the header's search opens. Without it the page reads the same, with the contents in full above
// the text.
import "./base.css";
import { countAfterLoad } from "./count.ts";
import { currentOf } from "./current.ts";
import { attachSearch } from "./search.ts";
import "./spec.css";

// The dev server links these stylesheets in the head too, for the first paint; the imports above have applied them now,
// and update them when they change.
for (const link of document.querySelectorAll("link[data-dev]")) link.remove();

const NARROW = matchMedia("(width < 960px)");
/** Wide enough for "On this page" beside the text (`spec.css`). */
const WIDE = matchMedia("(width >= 1200px)");

/** The id a link of the page's own sections goes to. */
const idOf = (link: HTMLAnchorElement): string => decodeURIComponent(link.hash.slice(1));

/**
 * Each subsection's section, as the contents list them, a section's `level-2` item followed by its subsections': which
 * group of "On this page" shows while it is read.
 */
function sectionsOf(toc: HTMLElement): Map<string, string> {
  const sections = new Map<string, string>();
  let section: string | undefined;
  for (const link of toc.querySelectorAll<HTMLAnchorElement>("li > a[href^='#']")) {
    if (link.parentElement?.classList.contains("level-2")) section = idOf(link);
    if (section !== undefined) sections.set(idOf(link), section);
  }
  return sections;
}

function scrollSpy(toc: HTMLElement, onPage: HTMLElement | null): void {
  const links = new Map<string, HTMLAnchorElement>([...toc.querySelectorAll<HTMLAnchorElement>("a[href^='#']")].map((link) => [idOf(link), link]));
  const pageLinks = [...(onPage?.querySelectorAll<HTMLAnchorElement>("a[href^='#']") ?? [])];
  // The contents list every section, in the page's order; "On this page" repeats some of them.
  const ids = [...links.keys()].filter((id) => document.getElementById(id));
  const sections = sectionsOf(toc);
  const groups = [...(onPage?.querySelectorAll<HTMLElement>("[data-section]") ?? [])];
  const header = Number.parseFloat(getComputedStyle(document.documentElement).scrollPaddingTop) || 80;
  let shownId: string | undefined;
  let queued = false;
  const update = () => {
    queued = false;
    const id = currentOf(ids, (heading) => document.getElementById(heading)?.getBoundingClientRect().top ?? 0, header + 8);
    if (id === shownId) return;
    shownId = id;
    for (const marked of document.querySelectorAll("a[aria-current='location']")) marked.removeAttribute("aria-current");
    for (const each of pageLinks) if (idOf(each) === id) each.setAttribute("aria-current", "location");
    // "On this page" shows the subsections of the section being read.
    const section = id === undefined ? undefined : sections.get(id);
    for (const group of groups) group.hidden = group.dataset.section !== section;
    // The contents mark the subsection read; on a wide screen, where "On this page" lists the subsections, its section.
    const link = id === undefined ? undefined : links.get((onPage && WIDE.matches ? section : undefined) ?? id);
    link?.setAttribute("aria-current", "location");
    // The menu's button, on a narrow screen, says where the reader is.
    const button = toc.querySelector(".toc-toggle");
    if (button) button.textContent = link ? `Contents: ${link.textContent}` : "Contents";
    // Keep it in view in the contents, which scroll on their own beside the text.
    if (link && !NARROW.matches) {
      const list = toc.querySelector("ol");
      if (list && (link.offsetTop < list.scrollTop || link.offsetTop > list.scrollTop + list.clientHeight - 40)) list.scrollTop = link.offsetTop - list.clientHeight / 3;
    }
  };
  addEventListener(
    "scroll",
    () => {
      if (queued) return;
      queued = true;
      requestAnimationFrame(update);
    },
    { passive: true },
  );
  // Across 1200px, what the contents mark changes with the place read.
  WIDE.addEventListener("change", () => {
    shownId = undefined;
    update();
  });
  update();
}

function menu(toc: HTMLElement): void {
  const button = toc.querySelector<HTMLButtonElement>(".toc-toggle");
  if (!button) return;
  const fold = () => {
    button.hidden = !NARROW.matches;
    toc.classList.toggle("folded", NARROW.matches && button.getAttribute("aria-expanded") !== "true");
  };
  button.addEventListener("click", () => {
    button.setAttribute("aria-expanded", String(button.getAttribute("aria-expanded") !== "true"));
    fold();
  });
  // A section chosen from the menu closes it.
  toc.addEventListener("click", (event) => {
    if (event.target instanceof HTMLAnchorElement && NARROW.matches) {
      button.setAttribute("aria-expanded", "false");
      fold();
    }
  });
  NARROW.addEventListener("change", fold);
  fold();
}

const toc = document.querySelector<HTMLElement>(".toc");
if (toc) {
  scrollSpy(toc, document.querySelector<HTMLElement>(".onpage"));
  menu(toc);
}
attachSearch(document);
countAfterLoad(window);
