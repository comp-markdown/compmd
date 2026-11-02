// `@compmd/registry`: packs from an npm registry, resolved, fetched and checked for the processor
// (docs/resolving-registry-packs.md).

export { defaultCache, memoryCache, type KeptVersions, type PackCache } from "./cache.ts";
export { registryProxy, type ProxyOptions } from "./proxy.ts";
export { isPackName, NpmRegistry, type NpmRegistryOptions, type Registry, type VersionInfo, type Versions } from "./registry.ts";
export { DEFAULT_MAX_PACK_SIZE, resolvePacks, type ModulePlace, type Problem, type ResolveOptions, type Resolved } from "./resolve.ts";
