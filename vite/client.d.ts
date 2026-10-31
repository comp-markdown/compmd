// The types of `virtual:compmd`: add `"@compmd/vite/client"` to `compilerOptions.types`, or reference it.

declare module "virtual:compmd" {
  import type { Request } from "@compmd/processor";
  import type { SandboxOptions } from "@compmd/sandbox";

  /** What the processor takes from the app: its packs, the ambient ones and the policy. Spread it into a request. */
  export const request: Required<Pick<Request, "packs" | "ambient" | "policy">>;

  /** What the sandbox takes from the app: where its frames are, its trusted components, where built modules are. */
  export const sandbox: Required<Pick<SandboxOptions, "base" | "trusted" | "modules">>;
}
