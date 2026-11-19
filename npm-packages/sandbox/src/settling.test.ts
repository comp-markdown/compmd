// @vitest-environment happy-dom
import { afterEach, describe, expect, it, vi } from "vitest";
import { stylesheetsLoaded } from "./settling.ts";

/** A stylesheet link in `document`'s head, still loading: happy-dom loads none. */
function loading(document: Document, rel = "stylesheet"): HTMLLinkElement {
  const link = document.createElement("link");
  link.setAttribute("rel", rel);
  Object.defineProperty(link, "sheet", { configurable: true, value: null });
  return document.head.appendChild(link);
}

/** Whether `promise` has settled, once what is queued has run. */
async function settled(promise: Promise<unknown>): Promise<boolean> {
  let done = false;
  void promise.then(() => (done = true));
  await new Promise((resolve) => setTimeout(resolve, 0));
  return done;
}

afterEach(() => {
  document.head.replaceChildren();
  vi.restoreAllMocks();
});

describe("stylesheetsLoaded", () => {
  it("never looks through the whole document, however many views settle in it", async () => {
    const scan = vi.spyOn(document, "querySelectorAll");
    for (let i = 0; i < 50; i++) await stylesheetsLoaded(document);
    expect(scan).not.toHaveBeenCalled();
  });

  it("waits for a stylesheet link still loading, one added after an earlier view settled too", async () => {
    await stylesheetsLoaded(document);
    const link = loading(document);
    const waiting = stylesheetsLoaded(document);
    expect(await settled(waiting)).toBe(false);
    link.dispatchEvent(new Event("load"));
    expect(await settled(waiting)).toBe(true);
  });

  it("waits for a stylesheet that fails as for one that loads, and for no other link", async () => {
    loading(document, "icon");
    loading(document, "preload");
    expect(await settled(stylesheetsLoaded(document))).toBe(true);
    const link = loading(document);
    const waiting = stylesheetsLoaded(document);
    link.dispatchEvent(new Event("error"));
    expect(await settled(waiting)).toBe(true);
  });
});
