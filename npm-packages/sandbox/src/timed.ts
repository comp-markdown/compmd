// A component's mount as a window's observer records it (docs/devtools.md, decision 6): mounted, with how long its
// code took, each update and its time, and unmounted.

import type { Mount, Params, Slots } from "@compmd/render";
import type { Observer } from "./observer.ts";

/**
 * `mount` with each mount and update timed by `observer`. `mount` is the component's own code, so that `mounted` is
 * recorded once that code has run. A mount that carries more than a function (`rendersSlots` marks one) is left as it
 * is: the renderer reads what it carries.
 *
 * @example
 * const mount = timed("stars", (target) => { target.textContent = "★★★★☆"; }, observer);
 */
export function timed(name: string, mount: Mount, observer: Observer): Mount {
  if (Object.getOwnPropertySymbols(mount).length > 0) return mount;
  return (target, context) => {
    const started = performance.now();
    const mounted = mount(target, context);
    const use = observer.mounted(name, performance.now() - started);
    return {
      ...(mounted?.update
        ? {
            update: (params: Params, slots: Slots) => {
              const updating = performance.now();
              mounted.update?.(params, slots);
              observer.updated(use, performance.now() - updating);
            },
          }
        : {}),
      destroy: () => {
        observer.unmounted(use);
        mounted?.destroy?.();
      },
    };
  };
}
