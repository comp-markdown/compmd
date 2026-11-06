// The tools' tabs: with a page, the page and the CompMD Inspector, which holds the views as tabs of its own; without one,
// as the inspector alone is, the Inspector's tabs only. Under the Inspector's views, the drawer: the Console, and the
// document's inputs when the tools can set them.

import { createRawSnippet, flushSync, mount, unmount } from "svelte";
import { afterEach, describe, expect, it } from "vitest";
import Panes from "./Panes.svelte";

const targets: HTMLElement[] = [];
const mounted: ReturnType<typeof mount>[] = [];
afterEach(() => {
  for (const component of mounted.splice(0)) void unmount(component);
  for (const target of targets.splice(0)) target.remove();
});

const INSPECTOR = ["Parse tree", "Expanded tree", "Data", "Components", "Sandbox", "Messages", "Plain Text", "Plain Markdown"];

interface Options {
  withPage?: boolean;
  text?: string;
  inputs?: Record<string, { type: string; value: unknown }>;
  onset?: (name: string, value: unknown) => void;
}

/**
 * The tools' panes, with a page when `withPage`, the document `text` when given, and `inputs` with `onset` when given.
 * No view is asked for: the views' worker isn't there.
 */
function panes(withPage: boolean, { text, inputs, onset }: Omit<Options, "withPage"> = {}): HTMLElement {
  const target = document.body.appendChild(document.createElement("div"));
  targets.push(target);
  const page = createRawSnippet(() => ({ render: () => '<p class="the-page">The page</p>' }));
  const props = {
    document: undefined,
    revision: 0,
    observed: [],
    dropped: {},
    entries: [],
    ...(withPage ? { page } : {}),
    ...(text === undefined ? {} : { text }),
    ...(inputs ? { inputs, onset: onset ?? (() => undefined) } : {}),
  };
  mounted.push(mount(Panes, { target, props: props as never }));
  flushSync();
  return target;
}

const click = (target: HTMLElement, selector: string) => {
  target.querySelector<HTMLElement>(selector)?.click();
  flushSync();
};

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
    expect(selected(target)).toEqual(["CompMD Inspector", "Parse tree", "Console"]);
    expect(target.querySelector<HTMLElement>("#view-page")?.hidden).toBe(true);
    expect(target.querySelector(".the-page")).not.toBeNull();
  });

  it("are the Inspector's alone without a page, as the inspector has them", () => {
    const target = panes(false);
    expect(tabs(target, "The page and the CompMD Inspector")).toEqual([]);
    expect(tabs(target, "The CompMD Inspector's views")).toEqual(INSPECTOR);
    target.querySelector<HTMLButtonElement>("#tab-sandbox")?.click();
    flushSync();
    expect(selected(target)).toEqual(["Sandbox", "Console"]);
  });

  it("show the document's text as written in Plain Text", () => {
    const target = panes(false, { text: "---\ntitle: Notes\n---\n\n:: box[Hi]\n" });
    click(target, "#tab-text");
    expect(target.querySelector("#view-text pre")?.textContent).toBe("---\ntitle: Notes\n---\n\n:: box[Hi]\n");
  });
});

describe("the drawer's Inputs", () => {
  const INPUTS = { region: { type: "string", value: "North" }, open: { type: "boolean", value: true } };

  it("is a tab beside the Console, each input with its control, and sets the input it changes", () => {
    const set: [string, unknown][] = [];
    const target = panes(true, { inputs: INPUTS, onset: (name, value) => set.push([name, value]) });
    click(target, "#tab-inspector");
    expect(tabs(target, "The drawer")).toEqual(["Console", "Inputs"]);
    click(target, "#drawer-inputs");
    const box = target.querySelector<HTMLInputElement>(".drawer input[type=checkbox]");
    expect(target.querySelector(".drawer .console")).toBeNull();
    expect([...target.querySelectorAll(".drawer .inputs code")].map((name) => name.textContent)).toEqual(["region", "open"]);
    box?.click();
    flushSync();
    expect(set).toEqual([["open", false]]);
  });

  it("isn't there when the tools have no inputs to set, as in the inspector", () => {
    const target = panes(false);
    expect(tabs(target, "The drawer")).toEqual(["Console"]);
  });

  it("isn't there for a document without inputs", () => {
    const target = panes(false, { inputs: {} });
    expect(tabs(target, "The drawer")).toEqual(["Console"]);
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
