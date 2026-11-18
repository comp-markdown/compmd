// The React binding (spec §7.6) for `jsx` and `tsx` components, `react@19`.
import { createElement, Fragment, type ComponentType, type ReactNode } from "react";
import { flushSync } from "react-dom";
import { createRoot } from "react-dom/client";
import type { Mount, MountContext, Params, Slots } from "../contract.ts";
import { camelCase, namedSlots, place, propsOf, slotElements } from "./place.ts";

/** The `compmd` prop: the host contract's parts that aren't params or slot nodes. */
export type CompmdProp = Pick<MountContext, "slots" | "set" | "fallback" | "headingLevel" | "scheme" | "fail" | "busy" | "host">;

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
 * `update` renders into the same root, so the component keeps its state. What it throws while rendering is thrown from
 * `mount` or `update`, and after it fails or throws, the next update renders it into a new root (spec §7.4).
 *
 * @example
 * function Card({ title, children }) { return <section><h3>{title}</h3>{children}</section>; }
 * createPage(root, { components: { card: react(Card) } });
 */
export function react(Component: ComponentType<Record<string, unknown>>): Mount {
  return (target, context) => {
    // What the component threw while rendering: React reports it rather than throwing, and the renderer needs the throw.
    let thrown: { error: unknown } | undefined;
    const takeThrown = () => {
      const taken = thrown;
      thrown = undefined;
      return taken;
    };
    const newRoot = () =>
      createRoot(target, {
        onUncaughtError: (error) => {
          thrown = { error };
        },
      });
    let root = newRoot();
    // The component failed or threw, so the renderer showed the fallback in `target` and empties it before the next
    // update: it renders into a new root then, since what it drew is gone.
    let broken = false;
    const fail = (message: string) => {
      broken = true;
      context.fail(message);
    };
    const render = (params: Params, slots: Slots) => {
      if (broken) {
        root.unmount();
        root = newRoot();
        broken = false;
      }
      const props: Record<string, unknown> = { ...propsOf(params), children: slotNode(slots.default) };
      for (const name of namedSlots(slots)) props[camelCase(name)] = slotNode(slots[name]);
      // `scheme` is read at each render: the renderer sets it before the update a change of scheme brings.
      const compmd: CompmdProp = { slots, set: context.set, fallback: context.fallback, headingLevel: context.headingLevel, scheme: context.scheme, fail, busy: context.busy, host: context.host };
      takeThrown();
      flushSync(() => {
        root.render(createElement(Component, { ...props, compmd }));
      });
      const failure = takeThrown();
      if (failure) {
        broken = true;
        throw failure.error;
      }
    };
    try {
      render(context.params, context.slots);
    } catch (error) {
      root.unmount();
      throw error;
    }
    return {
      update: render,
      destroy: () => {
        root.unmount();
      },
    };
  };
}
