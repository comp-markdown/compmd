// The host contract (spec §7.4): how the renderer runs a host-language component, whatever its framework.

/** A component's params, validated and typed by the processor. */
export type Params = Record<string, unknown>;

/** One block of a slot: the box the renderer made around it, and, when the block is a component, its name and params. */
export interface SlotItem {
  element: Element;
  component?: { name: string; params: Params };
}

/** One element of a template slot (spec §7.5), with its rendered template. */
export interface SlotInstance {
  value: unknown;
  items: SlotItem[];
}

/** Each slot's content, by slot name (`default` too). */
export type Slots = Record<string, SlotItem[] | SlotInstance[]>;

/** The invocation's fallback, as plain text and rendered. */
export interface Fallback {
  text: string;
  element: Element;
}

export interface MountContext {
  params: Params;
  slots: Slots;
  /** Changes the input bound to an `input<…>` param; refused for any other param, and during `update`. */
  set: (param: string, value: unknown) => void;
  fallback: Fallback;
  /** The level the component's top heading should have (spec §7.2). */
  headingLevel: number;
  /** What the host gives components it trusts; absent otherwise. */
  host?: unknown;
}

export interface Mounted {
  update?(params: Params, slots: Slots): void;
  destroy?(): void;
}

/**
 * A component as the renderer runs it: a plain component's default export (the plain binding, spec §7.6), or what
 * `react()` and `svelte()` make of a framework component.
 */
// A plain component that needs no update or destroy returns nothing, as in spec §7.6's examples.
// eslint-disable-next-line @typescript-eslint/no-invalid-void-type
export type Mount = (target: Element, context: MountContext) => Mounted | undefined | void;

/** Why a component shows its fallback: it isn't available here, or it threw while mounting or updating (W118). */
export interface ComponentError {
  component: string;
  reason: "missing" | "threw";
  error?: unknown;
}
