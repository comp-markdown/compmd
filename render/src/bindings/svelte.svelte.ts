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
 * After the component fails or throws while updating, the next update mounts it again (spec §7.4).
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
    // The component failed or threw, so the renderer showed the fallback in `target` and empties it before the next
    // update: the component is mounted again then, since what it drew is gone.
    let broken = false;
    const fail = (message: string) => {
      broken = true;
      context.fail(message);
    };
    const fill = (params: Params, slots: Slots) => {
      for (const [slot, content] of Object.entries(slots)) elements[slot] = slotElements(content);
      const next: Record<string, unknown> = { ...propsOf(params), children: snippetFor("default") };
      for (const name of namedSlots(slots)) next[camelCase(name)] = snippetFor(name);
      // `scheme` is read at each fill: the renderer sets it before the update a change of scheme brings.
      next.compmd = { slots, set: context.set, fallback: context.fallback, headingLevel: context.headingLevel, scheme: context.scheme, fail, busy: context.busy, host: context.host };
      for (const key of Object.keys(props)) if (!(key in next)) Reflect.deleteProperty(props, key);
      Object.assign(props, next);
    };
    const start = () => {
      const started = mount(Component, { target, props });
      try {
        flushSync();
      } catch (error) {
        void unmount(started);
        throw error;
      }
      return started;
    };
    fill(context.params, context.slots);
    let instance: ReturnType<typeof mount> | undefined = start();
    return {
      update(params, slots) {
        fill(params, slots);
        try {
          if (!broken) flushSync();
          else {
            if (instance) void unmount(instance);
            instance = undefined;
            instance = start();
            broken = false;
          }
        } catch (error) {
          broken = true;
          throw error;
        }
      },
      destroy() {
        if (instance) void unmount(instance);
      },
    };
  };
}
