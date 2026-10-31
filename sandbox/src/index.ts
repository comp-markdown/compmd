// @compmd/sandbox: CompMD components in a browser at the spec's trust levels (docs/sandbox.md), in four layers: the
// protocol (`@compmd/sandbox/protocol`), the frame runtime (in the frame documents), the host runtime, and defaults.
export { createSandbox, type Sandbox, type SandboxOptions, type View, type ViewEvents, type ViewSpliceOptions } from "./sandbox.ts";
export { HostRuntime, type HostEvents, type HostRuntimeOptions, type ModuleSource } from "./host/runtime.ts";
export { frameUse, type UseFrame, type UseFrameOptions } from "./use.ts";
export { hostClient, serveHost, type HostApi, type HostClient } from "./host-api.ts";
export { networkPolicy } from "./frame/narrow.ts";
