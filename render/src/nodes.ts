// A component that renders its slots itself, from their nodes, rather than taking the elements the renderer made of
// them: a sandbox's frame for a use, which renders the use and its slots inside the frame, at the use's trust level.

import type { Mount, MountContext } from "./contract.ts";
import type { ExpandedComponent, ExpandedInput } from "./tree.ts";

/** What a mount that renders its slots gets: the contract without rendered slots, the use's node and the inputs. */
export interface NodesContext extends MountContext {
  node: ExpandedComponent;
  inputs: Readonly<Record<string, ExpandedInput>>;
}

/** A mounted component that renders its slots: it gets the use's node again on each update. */
export interface NodesMounted {
  update?(node: ExpandedComponent, inputs: Readonly<Record<string, ExpandedInput>>): void;
  destroy?(): void;
}

// A mount that returns nothing, as a plain component may, is allowed here too.
// eslint-disable-next-line @typescript-eslint/no-invalid-void-type
export type NodesMount = (target: Element, context: NodesContext) => NodesMounted | undefined | void;

const NODES = Symbol("compmd.rendersSlots");

/**
 * `mount` as a component for `components` that renders its slots itself: the renderer renders none of the use's
 * slots, and gives `mount` the use's node, its slots in it, and the document's inputs.
 *
 * @example
 * const components = { map: rendersSlots((target, { node, inputs }) => frameFor(target, node, inputs)) };
 */
export function rendersSlots(mount: NodesMount): Mount {
  const marked: Mount = () => {
    throw new Error("a mount that renders its slots runs only through the renderer");
  };
  return Object.assign(marked, { [NODES]: mount });
}

/** The mount `rendersSlots` made `mount` from, if it did. */
export function nodesMountOf(mount: Mount | undefined): NodesMount | undefined {
  return (mount as { [NODES]?: NodesMount } | undefined)?.[NODES];
}

/**
 * `mount` as the component layer runs it: a mount that renders its slots gets the node and inputs `current` gives,
 * when it mounts and on each update.
 */
export function runnable(mount: Mount | undefined, current: () => { node: ExpandedComponent; inputs: Readonly<Record<string, ExpandedInput>> }): Mount | undefined {
  const nodes = nodesMountOf(mount);
  if (!nodes) return mount;
  return (target, context) => {
    const mounted = nodes(target, { ...context, ...current() });
    if (!mounted) return undefined;
    return {
      update: () => {
        const { node, inputs } = current();
        mounted.update?.(node, inputs);
      },
      destroy: () => mounted.destroy?.(),
    };
  };
}
