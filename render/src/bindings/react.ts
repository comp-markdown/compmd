// The React binding (spec §7.6) for `jsx` and `tsx` components, `react@19`.
import { createElement, Fragment, type ComponentType, type ReactNode } from "react";
import { flushSync } from "react-dom";
import { createRoot } from "react-dom/client";
import type { Mount, MountContext, Params, Slots } from "../contract.ts";
import { camelCase, namedSlots, place, propsOf, slotElements } from "./place.ts";

/** The `compmd` prop: the host contract's parts that aren't params or slot nodes. */
export type CompmdProp = Pick<MountContext, "slots" | "set" | "fallback" | "headingLevel" | "host">;

/** A stable React key per element, so an element keeps its placeholder when items before it change. */
const keys = new WeakMap<Element, number>();
let nextKey = 0;
function keyOf(element: Element): number {
  let key = keys.get(element);
  if (key === undefined) keys.set(element, (key = nextKey++));
  return key;
}

/** Holds one of the renderer's elements, placed once and never moved. */
function Placed({ element }: { element: Element }): ReactNode {
  const ref = (div: HTMLDivElement | null) => {
    if (div) place(div, [element]);
  };
  return createElement("div", { style: { display: "contents" }, ref });
}

function slotNode(slot: Slots[string] | undefined): ReactNode {
  return createElement(Fragment, null, ...slotElements(slot).map((element) => createElement(Placed, { key: keyOf(element), element })));
}

/**
 * Runs a React component through the host contract: params are props of the same name in camelCase, the default slot
 * is `children`, a named slot is a prop of its name, and the rest is the `compmd` prop. Rendering is synchronous, and
 * `update` renders into the same root, so the component keeps its state.
 *
 * @example
 * function Card({ title, children }) { return <section><h3>{title}</h3>{children}</section>; }
 * createPage(root, { components: { card: react(Card) } });
 */
export function react(Component: ComponentType<Record<string, unknown>>): Mount {
  return (target, context) => {
    const root = createRoot(target);
    const render = (params: Params, slots: Slots) => {
      const props: Record<string, unknown> = { ...propsOf(params), children: slotNode(slots.default) };
      for (const name of namedSlots(slots)) props[camelCase(name)] = slotNode(slots[name]);
      const compmd: CompmdProp = { slots, set: context.set, fallback: context.fallback, headingLevel: context.headingLevel, host: context.host };
      flushSync(() => {
        root.render(createElement(Component, { ...props, compmd }));
      });
    };
    render(context.params, context.slots);
    return {
      update: render,
      destroy: () => {
        root.unmount();
      },
    };
  };
}
