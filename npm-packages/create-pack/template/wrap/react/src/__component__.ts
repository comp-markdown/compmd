// `__component__`: __from__'s component in the plain binding (spec §7.6), written once by `npm create @compmd/pack --
// --from __from__` and yours to change: `compmd-pack build` writes the signature in `index.comp.md` at each build, never
// this file. React and React DOM stay imports, which the frames' import map gives; __from__ is bundled in.
import { createElement, type ComponentProps } from "react";
import { flushSync } from "react-dom";
import { createRoot } from "react-dom/client";
import { component, fitBox } from "@compmd/pack/component";
import __imported__ from "__from__";

/** The params: the props `compmd.wrap.json` lists, as the generated signature declares them. */
type Params = ComponentProps<typeof Wrapped>;

export default component<Params>((target, context) => {
  const root = createRoot(target);
  /** The props for `next`: the params__label_note__. */
  const propsOf = (next: Params): Params => __props__;
  const draw = (next: Params) => {
    // Drawn at once, so that its drawing fits the box (`object-fit`, `object-position`): as large as the box allows.
    flushSync(() => {
      root.render(createElement(Wrapped, propsOf(next)));
    });
    const drawing = target.firstElementChild;
    if (drawing) fitBox(drawing, context.fit);
  };
  draw(context.params);
  return {
    update: draw,
    destroy: () => {
      root.unmount();
    },
  };
});
