// The React and Svelte bindings (spec §7.6), through the page: params as props in camelCase, slots as children and
// named props, the rest in `compmd`, and state kept through an update. A slot's element keeps its focus.
import { createElement, useState, type ReactNode } from "react";
import { flushSync } from "react-dom";
import { afterEach, describe, expect, it } from "vitest";
import { react } from "../src/bindings/react.ts";
import { svelte } from "../src/bindings/svelte.svelte.ts";
import type { Mount } from "../src/contract.ts";
import { createPage } from "../src/page.ts";
import type { ExpandedRoot } from "../src/tree.ts";
import Card from "./Card.svelte";

function ReactCard({ title, fontSize, children, footer, compmd }: { title: string; fontSize: string; children: ReactNode; footer: ReactNode; compmd: { headingLevel: number } }) {
  const [clicks, setClicks] = useState(0);
  return createElement(
    "section",
    null,
    createElement("h3", { "data-level": compmd.headingLevel, "data-size": fontSize }, title),
    createElement("button", {
      onClick: () => {
        setClicks((n) => n + 1);
      },
    }, String(clicks)),
    children,
    footer,
  );
}

const card = (title: string): ExpandedRoot => ({
  type: "root",
  children: [
    { type: "heading", depth: 1, children: [{ type: "text", value: "Page" }] },
    {
      type: "componentContainer",
      name: "card",
      params: [
        { key: "title", breakpoint: null, value: title },
        { key: "font-size", breakpoint: null, value: "small" },
      ],
      children: [{ type: "componentLeaf", name: "notes", params: [], fallback: [] }],
      slots: [{ type: "componentSlot", name: "footer", children: [{ type: "paragraph", children: [{ type: "text", value: "Footer" }] }] }],
    },
  ],
});

const notes: Mount = (target) => {
  target.appendChild(target.ownerDocument.createElement("textarea"));
};

let root: HTMLElement;
afterEach(() => {
  root.remove();
});

describe.each([
  ["React", react(ReactCard as never)],
  ["Svelte", svelte(Card as never)],
])("the %s binding", (_name, mount) => {
  it("passes params, slots and the heading level, and keeps state and focus through an update", () => {
    root = document.body.appendChild(document.createElement("main"));
    const page = createPage(root, { components: { card: mount, notes } });
    page.render(card("One"));
    const heading = root.querySelector("h3")!;
    expect(heading.textContent).toBe("One");
    expect(heading.dataset.level).toBe("2");
    expect(root.querySelector("section")?.textContent).toContain("Footer");

    const button = root.querySelector("button")!;
    flushSync(() => {
      button.click();
    });
    const area = root.querySelector("textarea")!;
    area.focus();
    page.render(card("Two"));

    expect(root.querySelector("h3")?.textContent).toBe("Two");
    expect(root.querySelector("button")?.textContent).toBe("1");
    expect(document.activeElement).toBe(area);
    page.destroy();
  });
});

it("gives React a param with a dash in camelCase", () => {
  root = document.body.appendChild(document.createElement("main"));
  createPage(root, { components: { card: react(ReactCard as never), notes } }).render(card("One"));
  expect(root.querySelector("h3")?.dataset.size).toBe("small");
});
