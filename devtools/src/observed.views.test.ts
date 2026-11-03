// The observer tabs show what frames send only as text (docs/devtools.md, O1): a message, a fallback's message, a
// component's name and a frame's claim, each written by code of a frame's level, never become markup or a link.

import type { ObservedEvent } from "@compmd/sandbox";
import { flushSync, mount, unmount } from "svelte";
import { afterEach, describe, expect, it } from "vitest";
import { consoleOf } from "./console.ts";
import ConsoleView from "./ConsoleView.svelte";
import { treeOf } from "./frames.ts";
import { timelineOf } from "./messages.ts";
import MessagesView from "./MessagesView.svelte";
import SandboxView from "./SandboxView.svelte";

const MARKUP = '<img src="x" onerror="globalThis.ran = true"><a href="javascript:alert(1)">open</a>';

/** What a frame of the page could send: markup and a script's URL wherever it writes text. */
const EVENTS: ObservedEvent[] = [
  { type: "frame", frame: "1", level: "strict", renders: { kind: "use", component: MARKUP }, at: 1 },
  { type: "message", frame: "1", way: "up", data: { type: "error", reason: "threw", component: MARKUP, message: MARKUP, url: "javascript:alert(1)" }, at: 2 },
  { type: "mounted", frame: "1", use: 1, component: MARKUP, ms: 1, at: 3, by: "1" },
  { type: "fallback", frame: "1", component: MARKUP, reason: "threw", message: MARKUP, at: 4, by: "1" },
  { type: "refused", frame: "1", input: MARKUP, at: 5 },
  { type: "frame", frame: "1", level: "permissive", renders: { kind: "use", component: MARKUP }, network: [MARKUP], at: 6, by: "1" },
];

const targets: HTMLElement[] = [];
const mounted: ReturnType<typeof mount>[] = [];
afterEach(() => {
  for (const component of mounted.splice(0)) void unmount(component);
  for (const target of targets.splice(0)) target.remove();
});

/** A place on the page for a view. */
function place(): HTMLElement {
  const target = document.body.appendChild(document.createElement("div"));
  targets.push(target);
  return target;
}

/** That `target` shows the markup as text (`shown`, quoted in a tree), with no element or link made of it. */
function showsAsText(target: HTMLElement, shown = MARKUP): void {
  expect(target.textContent).toContain(shown);
  expect(target.querySelector("img")).toBeNull();
  expect(target.querySelector("a")).toBeNull();
  expect((globalThis as { ran?: boolean }).ran).toBeUndefined();
}

describe("the observer tabs, with what a frame wrote", () => {
  it("show a message's copy as text, in the Messages tab, opened", () => {
    const target = place();
    mounted.push(mount(MessagesView, { target, props: { timeline: timelineOf(EVENTS) } }));
    target.querySelector<HTMLButtonElement>(".message .toggle")?.click();
    flushSync();
    expect(target.textContent).toContain("javascript:alert(1)");
    showsAsText(target, JSON.stringify(MARKUP));
  });

  it("show a frame's components, fallbacks and claims as text, in the Sandbox tab", () => {
    const target = place();
    mounted.push(mount(SandboxView, { target, props: { tree: treeOf(EVENTS), dropped: {} } }));
    showsAsText(target);
    expect(target.querySelector(".claim")?.textContent).toContain("reported by 1");
  });

  it("show fallbacks and refused inputs as text, in the Console", () => {
    const target = place();
    mounted.push(mount(ConsoleView, { target, props: { entries: consoleOf([], EVENTS), onselect: () => undefined } }));
    showsAsText(target);
    expect(target.querySelectorAll(".reported")).toHaveLength(1);
  });
});
