// @compmd/sandbox: CompMD components in a browser at the spec's trust levels
// (docs/running-components-in-the-sandbox.md), in four layers: the protocol (`@compmd/sandbox/protocol`), the frame
// runtime (in the frame documents), the host runtime, and defaults.
export { createSandbox, type Sandbox, type SandboxOptions, type View, type ViewEvents, type ViewSpliceOptions } from "./sandbox.ts";
export { diagnosticOf, type SandboxDiagnostic } from "./protocol/messages.ts";
export { HostRuntime, type HostEvents, type HostRuntimeOptions, type ModuleSource } from "./host/runtime.ts";
export type { BundleModuleSource } from "./found.ts";
export { frameUse, type UseFrame, type UseFrameOptions } from "./use.ts";
export { hostClient, localHost, serveHost, type HostApi, type HostClient } from "./host-api.ts";
export { networkPolicy } from "./frame/narrow.ts";
export { shikiHighlight } from "./highlight/highlight.ts";
export {
  allowOrigins,
  askWithDialog,
  loadData,
  localAnswers,
  originQuestion,
  type AllowOriginsOptions,
  type AskOrigin,
  type LoadDataOptions,
  type LoadedData,
  type OriginAnswer,
  type OriginAnswers,
} from "./origins.ts";
export { Observer } from "./observer.ts";
export type { Observation, ObservedEvent } from "./observation.ts";
export type { FrameId, Gone, Observed, Renders } from "./protocol/observed.ts";
