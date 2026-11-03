// The host contract (spec §7.4): how the renderer runs a host-language component, whatever its framework.

/** A component's params, validated and typed by the processor. */
export type Params = Record<string, unknown>;

/**
 * One block of a slot: the box the renderer made around it, and, when the block is a component, its name and params.
 * A component still on its way has `pending: true` and no params (spec §7.4): its item shows its fallback until it
 * arrives.
 */
export interface SlotItem {
  element: Element;
  component?: { name: string; params: Params; pending?: true };
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

/** A colour scheme a component is shown in. */
export type Scheme = "light" | "dark";

export interface MountContext {
  params: Params;
  slots: Slots;
  /** Changes the input bound to an `input<…>` param; refused for any other param, and during `update`. */
  set: (param: string, value: unknown) => void;
  fallback: Fallback;
  /** The level the component's top heading should have (spec §7.2). */
  headingLevel: number;
  /** The colour scheme the component is shown in; when it changes, the renderer sets it and calls `update`. */
  scheme: Scheme;
  /**
   * The component can't draw what it was given, found after `mount` or `update` returned: W118 with `message`, and the
   * fallback in `target` until the next `update`. Not for a resource that may come back.
   */
  fail: (message: string) => void;
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

/**
 * Why a component shows its fallback: it isn't available here, or it threw while mounting or updating or called
 * `fail`, with the message as `error` (W118).
 */
export interface ComponentError {
  component: string;
  reason: "missing" | "threw";
  error?: unknown;
}

/**
 * A region whose blocks use breakpoints has no container around it to follow, so they never apply: the caller gives
 * it one (a page, or an element with `compmd-container`).
 */
export interface LayoutError {
  reason: "no-container";
  message: string;
}

/** What `onError` reports. */
export type RenderError = ComponentError | LayoutError;
