// @compmd/sandbox/build: the frame documents for a host's origin, and the libraries its frames share (Node, at build).
export { buildFrames, type FramesOptions } from "./frames.ts";
export { buildLibraries, RUNTIMES, type LibrariesOptions, type Library } from "./libraries.ts";
export { permissivePolicy, strictPolicy } from "./policy.ts";
