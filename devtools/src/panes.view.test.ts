// The tools' tabs: with a page, the page and the CompMD Inspector, which holds the views as tabs of its own; without one,
// as the inspector alone is, the Inspector's tabs only.

import { createRawSnippet, flushSync, mount, unmount } from "svelte";
import { afterEach, describe, expect, it } from "vitest";
import Panes from "./Panes.svelte";

const targets: HTMLElement[] = [];
const mounted: ReturnType<typeof mount>[] = [];
afterEach(() => {
  for (const component of mounted.splice(0)) void unmount(component);
  for (const target of targets.splice(0)) target.remove();
});

const INSPECTOR = ["Plain Markdown", "Parse tree", "Expanded tree", "Data", "Components", "Sandbox", "Messages"];

/** The tools' panes, with a page when `withPage`; no document, so no view is asked for. */
function panes(withPage: boolean): HTMLElement {
  const target = document.body.appendChild(document.createElement("div"));
  targets.push(target);
  const page = createRawSnippet(() => ({ render: () => '<p class="the-page">The page</p>' }));
  mounted.push(mount(Panes, { target, props: { document: undefined, revision: 0, observed: [], dropped: {}, entries: [], ...(withPage ? { page } : {}) } }));
  flushSync();
  return target;
}

const tabs = (target: HTMLElement, label: string) => [...target.querySelectorAll(`[role=tablist][aria-label="${label}"] [role=tab]`)].map((tab) => tab.textContent);
const selected = (target: HTMLElement) => [...target.querySelectorAll("[role=tab][aria-selected=true]")].map((tab) => tab.textContent);

describe("the tools' tabs", () => {
  it("are the page and the CompMD Inspector with a page, the page first and kept mounted under the Inspector", () => {
    const target = panes(true);
    expect(tabs(target, "The page and the CompMD Inspector")).toEqual(["Page", "CompMD Inspector"]);
    expect(selected(target)).toEqual(["Page"]);
    expect(tabs(target, "The CompMD Inspector's views")).toEqual([]);
    target.querySelector<HTMLButtonElement>("#tab-inspector")?.click();
    flushSync();
    expect(tabs(target, "The CompMD Inspector's views")).toEqual(INSPECTOR);
    expect(selected(target)).toEqual(["CompMD Inspector", "Plain Markdown"]);
    expect(target.querySelector<HTMLElement>("#view-page")?.hidden).toBe(true);
    expect(target.querySelector(".the-page")).not.toBeNull();
  });

  it("are the Inspector's alone without a page, as the inspector has them", () => {
    const target = panes(false);
    expect(tabs(target, "The page and the CompMD Inspector")).toEqual([]);
    expect(tabs(target, "The CompMD Inspector's views")).toEqual(INSPECTOR);
    target.querySelector<HTMLButtonElement>("#tab-sandbox")?.click();
    flushSync();
    expect(selected(target)).toEqual(["Sandbox"]);
  });
});

describe("the Console's drawer", () => {
  it("is under the Inspector's views, whichever shows, and not under the page", () => {
    const target = panes(true);
    expect(target.querySelector(".drawer")).toBeNull();
    target.querySelector<HTMLButtonElement>("#tab-inspector")?.click();
    flushSync();
    const drawer = target.querySelector(".inspector-pane > .drawer");
    expect(drawer?.querySelector(".console")).not.toBeNull();
    target.querySelector<HTMLButtonElement>("#tab-data")?.click();
    flushSync();
    expect(target.querySelector(".inspector-pane > .drawer .console")).not.toBeNull();
  });

  it("hides and shows with its bar's button, and with Esc", () => {
    const target = panes(false);
    const toggle = target.querySelector<HTMLButtonElement>(".drawer-toggle");
    toggle?.click();
    flushSync();
    expect([toggle?.getAttribute("aria-expanded"), target.querySelector(".drawer .console")]).toEqual(["false", null]);
    target.querySelector(".inspector-pane")?.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true, cancelable: true }));
    flushSync();
    expect(target.querySelector(".drawer .console")).not.toBeNull();
  });

  it("grows and shrinks with the arrow keys on its handle", () => {
    const target = panes(false);
    const handle = target.querySelector<HTMLElement>(".drawer-handle");
    const height = () => Number(handle?.getAttribute("aria-valuenow"));
    const before = height();
    handle?.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowUp", bubbles: true, cancelable: true }));
    flushSync();
    expect(height()).toBeGreaterThan(before);
    handle?.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true, cancelable: true }));
    flushSync();
    expect(height()).toBe(before);
  });
});
