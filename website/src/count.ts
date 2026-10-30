// Visits counted with GoatCounter (website/docs/design.md, decision 8): page views, referrers, browsers and countries,
// with no cookies and nothing that identifies a reader. Its script loads once the page has loaded, so the page paints
// without it, and only on the published site, so the dev server and `pnpm check` count nothing.

/** Where visits are counted: the author's GoatCounter site. */
export const COUNTER = "https://hakanserce.goatcounter.com/count";

/** GoatCounter's script. */
export const SCRIPT = "https://gc.zgo.at/count.js";

const LOCAL = new Set(["localhost", "127.0.0.1", "[::1]", ""]);

/** Adds GoatCounter's script to `document`, unless the page is served from this machine. */
export function count(document: Document): void {
  if (LOCAL.has(document.location.hostname)) return;
  const script = document.createElement("script");
  script.async = true;
  script.dataset.goatcounter = COUNTER;
  script.src = SCRIPT;
  document.head.append(script);
}

/** Counts the visit once the page has loaded. */
export function countAfterLoad(window: Window): void {
  if (window.document.readyState === "complete") count(window.document);
  else
    window.addEventListener(
      "load",
      () => {
        count(window.document);
      },
      { once: true },
    );
}
