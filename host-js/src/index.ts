// @compmd/host: the serving contract of a CompMD host over the Fetch API (docs/hosting.md, decision 8). A bundle in a
// folder, for Node, is `@compmd/host/node`'s.
export { respond, SERVED_HEADERS, type Bundle, type Place, type Store } from "./respond.ts";
export { contentType } from "./content-type.ts";
