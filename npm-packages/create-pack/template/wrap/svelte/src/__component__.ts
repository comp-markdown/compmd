// `__component__`: __from__'s component in the plain binding (spec §7.6), written once by `npm create @compmd/pack --
// --from __from__` and yours to change: `compmd-pack build` writes the signature in `index.comp.md` at each build, never
// this file. Svelte stays an import, which the frames' import map gives; __from__, compiled from its sources with that
// Svelte, is bundled in.
import { mount, unmount, type ComponentProps } from "svelte";
import { component, fillBox } from "@compmd/pack/component";
import __imported__ from "__from__";

/** The params: the props `compmd.wrap.json` lists, as the generated signature declares them. */
type Params = ComponentProps<typeof Wrapped>;

export default component<Params>((target, context) => {
  // A box of its own, which the component takes its size from: the use's box's, or CSS's 300 by 150 without one.
  const box = target.appendChild(fillBox(document.createElement("div")));
  /** The props for `next`: the params__label_note__. */
  const propsOf = (next: Params): Params => __props__;
  // A plain module can't hold Svelte's `$state`, so new params mount the component again.
  let mounted = mount(Wrapped, { target: box, props: propsOf(context.params) });
  return {
    update: (next) => {
      void unmount(mounted);
      mounted = mount(Wrapped, { target: box, props: propsOf(next) });
    },
    destroy: () => {
      void unmount(mounted);
      box.remove();
    },
  };
});
