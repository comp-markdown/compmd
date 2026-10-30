// The spec page's script, on top of its static HTML: the contents mark the section being read, and fold into a menu on
// a narrow screen. Without it the page reads the same, with the contents in full above the text.
import "./base.css";
import { countAfterLoad } from "./count.ts";
import { currentOf } from "./current.ts";
import "./spec.css";

const NARROW = matchMedia("(width < 960px)");

function scrollSpy(toc: HTMLElement): void {
  const links = new Map([...toc.querySelectorAll<HTMLAnchorElement>("a[href^='#']")].map((link) => [decodeURIComponent(link.hash.slice(1)), link]));
  const ids = [...links.keys()].filter((id) => document.getElementById(id));
  const header = Number.parseFloat(getComputedStyle(document.documentElement).scrollPaddingTop) || 80;
  let shown: HTMLAnchorElement | undefined;
  let queued = false;
  const update = () => {
    queued = false;
    const id = currentOf(ids, (heading) => document.getElementById(heading)?.getBoundingClientRect().top ?? 0, header + 8);
    const link = id === undefined ? undefined : links.get(id);
    if (link === shown) return;
    shown?.removeAttribute("aria-current");
    link?.setAttribute("aria-current", "location");
    shown = link;
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
  scrollSpy(toc);
  menu(toc);
}
countAfterLoad(window);
