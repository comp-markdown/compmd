// What the bindings share (spec §7.6): a slot's elements, placed without moving them, and params as props.
import type { Params, SlotInstance, SlotItem, Slots } from "../contract.ts";

/** A slot's elements in order: its items, or every instance's items. */
export function slotElements(slot: readonly (SlotItem | SlotInstance)[] | undefined): Element[] {
  return (slot ?? []).flatMap((entry) => ("items" in entry ? entry.items.map((item) => item.element) : [entry.element]));
}

/**
 * Puts `elements` into `parent` in order, moving only those out of place, so an element already where it belongs
 * keeps its focus and selection: an element moved in the DOM loses them.
 */
export function place(parent: Element, elements: readonly Element[]): void {
  elements.forEach((element, i) => {
    const at = parent.childNodes[i];
    if (at !== element) parent.insertBefore(element, at ?? null);
  });
  while (parent.childNodes.length > elements.length) parent.lastChild?.remove();
}

/** `align-items` is `alignItems`: params as props. */
export function camelCase(name: string): string {
  return name.replace(/-([a-z])/g, (_, letter: string) => letter.toUpperCase());
}

/** A component's params as framework props. */
export function propsOf(params: Params): Params {
  return Object.fromEntries(Object.entries(params).map(([name, value]) => [camelCase(name), value]));
}

/** Slot names other than `default`, which frameworks get as props of the same name. */
export function namedSlots(slots: Slots): string[] {
  return Object.keys(slots).filter((name) => name !== "default");
}
