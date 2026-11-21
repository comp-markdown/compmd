// `__component__`: the `<__export__>` element of __from__ in the plain binding (spec §7.6), written once by `npm create
// @compmd/pack -- --from __from__` and yours to change: `compmd-pack build` writes the signature in `index.comp.md` at
// each build, never this file. Importing the package defines the element; it is bundled in.
import "__from__";
import { component } from "@compmd/pack/component";

/** The params: the element's properties `compmd.wrap.json` lists, as the generated signature declares them. */
type Params = Partial<Record<string, unknown>>;

/** A property's attribute: its name in kebab case, unless the element says otherwise. */
const attributeOf = (name: string): string => name.replace(/[A-Z]/g, (letter) => `-${letter.toLowerCase()}`);

export default component<Params>((target, context) => {
  const element = document.createElement("__export__");
  // An element reports what it couldn't draw as an `error` event, not a throw: the host then shows the fallback.
  element.addEventListener("error", (event) => {
    context.fail(event instanceof ErrorEvent && event.message ? event.message : "__export__ couldn't draw what it was given");
  });
  // It fills its box, which the use sizes.
  element.style.cssText = "display: block; width: 100%; height: 100%";
  /** The params given last, so a param left out next time is taken off. */
  let given: Params = {};
  /** The props for `next`: the params__label_note__. */
  const propsOf = (next: Params): Params => __props__;
  const draw = (next: Params) => {
    const props = propsOf(next);
    for (const name of new Set([...Object.keys(given), ...Object.keys(props)])) {
      const value = props[name];
      // A list or an object is a property: an attribute holds text.
      if (typeof value === "object" && value !== null) Object.assign(element, { [name]: value });
      else if (value === undefined || value === null || value === false) element.removeAttribute(attributeOf(name));
      else element.setAttribute(attributeOf(name), value === true ? "" : String(value as string | number));
    }
    given = props;
  };
  draw(context.params);
  target.append(element);
  return {
    update: draw,
    destroy: () => {
      element.remove();
    },
  };
});
