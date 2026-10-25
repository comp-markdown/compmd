// The Svelte binding (spec §7.6) for `svelte` components, `svelte@5`. It uses runes, so the caller's build compiles it
// with Svelte, as for any Svelte library.
import { createRawSnippet, flushSync, mount, unmount, type Component, type Snippet } from "svelte";
import type { Mount, Params, Slots } from "../contract.ts";
import { camelCase, namedSlots, place, propsOf, slotElements } from "./place.ts";

/**
 * Runs a Svelte 5 component through the host contract: params are props of the same name in camelCase, the default
 * slot is the `children` snippet, a named slot is a snippet prop of its name, and the rest is the `compmd` prop.
 * Props are state, so `update` changes them in place; each slot is one snippet for the component's life, placing the
 * slot's current elements. Changes are flushed, so the target holds the component when `mount` and `update` return.
 *
 * @example
 * <script>let { title, children } = $props();</script>
 * <section><h3>{title}</h3>{@render children?.()}</section>
 * <!-- createPage(root, { components: { card: svelte(Card) } }) -->
 */
export function svelte(Component: Component<Record<string, unknown>>): Mount {
  return (target, context) => {
    const elements: Record<string, Element[]> = $state({});
    const snippets = new Map<string, Snippet>();
    const snippetFor = (slot: string): Snippet => {
      let snippet = snippets.get(slot);
      if (!snippet) {
        snippet = createRawSnippet(() => ({
          render: () => `<div style="display: contents"></div>`,
          setup: (div) => {
            $effect(() => {
              place(div, elements[slot] ?? []);
            });
          },
        }));
        snippets.set(slot, snippet);
      }
      return snippet;
    };
    const props: Record<string, unknown> = $state({});
    const fill = (params: Params, slots: Slots) => {
      for (const [slot, content] of Object.entries(slots)) elements[slot] = slotElements(content);
      const next: Record<string, unknown> = { ...propsOf(params), children: snippetFor("default") };
      for (const name of namedSlots(slots)) next[camelCase(name)] = snippetFor(name);
      next.compmd = { slots, set: context.set, fallback: context.fallback, headingLevel: context.headingLevel, host: context.host };
      for (const key of Object.keys(props)) if (!(key in next)) Reflect.deleteProperty(props, key);
      Object.assign(props, next);
    };
    fill(context.params, context.slots);
    const instance = mount(Component, { target, props });
    flushSync();
    return {
      update(params, slots) {
        fill(params, slots);
        flushSync();
      },
      destroy() {
        void unmount(instance);
      },
    };
  };
}
