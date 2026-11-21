// Where a sandbox's frames and built modules are (docs/hosting.md, decisions 5 and 9): the bundle's paths, which the
// page's own build gives (`virtual:compmd`'s `sandbox`), under the origin the host's adapter gives; or, for a host that
// serves frames it built itself, their `base`.

import type { HostComponent } from "@compmd/processor";
import { givenOrigin } from "./given-origin.ts";
import type { ModuleSource } from "./host/runtime.ts";

type Place = NonNullable<HostComponent["module"]>;

/**
 * A built module: where it is served, by a full URL or by its path under the bundle's origin (`packs/<place>`, as the
 * bundle and the host's store serve it), with its file's hash; or its file as a `Blob` the host has checked.
 * `undefined` when the host has none.
 */
export type BundleModuleSource = (module: Place) => { url: string; integrity: string } | { path: string; integrity: string } | { code: Blob } | undefined;

/** Where the frames are, as a sandbox is given them. */
export interface Where {
  /** Where the bundle is served, a full URL or one relative to the page; without it, the adapter's `__COMPMD__.origin`. */
  origin?: string;
  /** The bundle's frames under the origin, `frames/<hash>/`, as `virtual:compmd` gives them. */
  frames?: string;
  /** Where the frame documents are, ending in `/`, in place of `origin` and `frames`: for frames a host built itself. */
  base?: string;
  modules?: BundleModuleSource;
}

/** Why a sandbox has no frames to use. */
export const NO_FRAMES = "createSandbox needs where its frames are: the bundle's `frames` (`virtual:compmd`'s `sandbox`) with its origin (an adapter's `__COMPMD__`), or a `base`";

/**
 * `href` as the page links to it: from the root of the page's own origin when it is on it, so the image gate takes a
 * pack's image for the host's own, as a relative one (spec §15), and never asks the reader about it.
 */
function fromThisPage(href: string | undefined): string | undefined {
  if (href === undefined) return undefined;
  const url = new URL(href);
  return url.origin === location.origin ? `${url.pathname}${url.search}${url.hash}` : href;
}

/** The URL of a pack's file by its place (`@acme/kit@1.4.2/icons/star.svg`), as the renderer's `packFile` asks it. */
export type PackFile = (place: string) => string | undefined;

/**
 * `given` with the frames' `base`, `modules` with each path under the origin made a URL, and, with an origin,
 * `packFile`: a pack's file under it, at `packs/<place>`, where the bundle and the host's store serve them.
 *
 * @throws Error when neither the bundle's frames with an origin nor a `base` say where the frames are.
 */
export function found<T extends Where>(given: T): Omit<T, "modules"> & { base: string; modules?: ModuleSource; packFile?: PackFile } {
  const { modules, ...rest } = given;
  const origin = given.origin ?? givenOrigin();
  const at = (path: string): string | undefined => (origin === undefined ? undefined : new URL(path, new URL(origin.replace(/\/*$/, "/"), location.href)).href);
  const base = given.base ?? (given.frames === undefined ? undefined : at(given.frames));
  if (base === undefined) throw new Error(NO_FRAMES);
  const packs = origin === undefined ? {} : { packFile: (place: string) => fromThisPage(at(`packs/${place}`)) };
  if (!modules) return { ...rest, base, ...packs };
  return {
    ...rest,
    base,
    ...packs,
    modules: (module) => {
      const source = modules(module);
      if (!source || !("path" in source)) return source;
      const url = at(source.path);
      return url === undefined ? undefined : { url, integrity: source.integrity };
    },
  };
}
