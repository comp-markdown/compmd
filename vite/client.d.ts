// The types of `virtual:compmd`: add `"@compmd/vite/client"` to `compilerOptions.types`, or reference it.

declare module "virtual:compmd" {
  import type { Request } from "@compmd/processor";
  import type { ResolveOptions } from "@compmd/registry";
  import type { SandboxOptions } from "@compmd/sandbox";

  /** What the processor takes from the app: its packs, the ambient ones and the policy. Spread it into a request. */
  export const request: Required<Pick<Request, "packs" | "ambient" | "policy">>;

  /**
   * What the sandbox takes from the app: the bundle's frames, its trusted components and where built modules are, each
   * a path under the bundle's origin: the app's `compmd/` folder (`origin`), or, for a bundle written to `out`, the
   * adapter's `globalThis.__COMPMD__.origin`, which the sandbox reads.
   */
  export const sandbox: Required<Pick<SandboxOptions, "frames" | "trusted" | "modules">> & Pick<SandboxOptions, "origin">;

  /**
   * What `resolvePacks` takes from the app: its registries (the app's proxy, with `fetch: "host"`), who fetches, and
   * its size cap. Spread it into the options; `undefined` when the configuration names no registry.
   */
  export const registry: (Required<Pick<ResolveOptions, "registries" | "fetch">> & Pick<ResolveOptions, "maxPackSize">) | undefined;
}
