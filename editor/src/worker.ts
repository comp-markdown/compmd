// The editor's worker: processing never runs in the text area's input handler, so each key press is painted first. It
// also fetches the packs a document imports from the registries the page says the playground's configuration names.

import { init, OpenDocumentJson } from "@compmd/processor";
import { defaultCache, NpmRegistry, resolvePacks } from "@compmd/registry";
import { Processing } from "./processing.ts";
import type { FromWorker, RegistryConfig, ToWorker } from "./protocol.ts";

const ready = init();
const post = (message: FromWorker) => {
  postMessage(message);
};
const processing = new Processing((request) => new OpenDocumentJson(request), {
  onFetched: (fetched) => {
    post({ type: "fetched", ...fetched });
  },
});

/** Fetches missing packs from `config`'s registries, keeping them across visits. */
function useRegistries({ registries, fetch, maxPackSize }: RegistryConfig): void {
  const options = {
    registries: Object.fromEntries(Object.entries(registries).map(([scope, url]) => [scope, new NpmRegistry({ url })])),
    fetch,
    ...(maxPackSize === undefined ? {} : { maxPackSize }),
    cache: defaultCache(),
  };
  processing.useRegistry((request) => resolvePacks(request, options));
}

addEventListener("message", ({ data }: MessageEvent<ToWorker | RegistryConfig>) => {
  if (data.type === "registry") {
    useRegistries(data);
    return;
  }
  // Messages wait for the module in the order they came.
  void ready.then(() => {
    post(processing.handle(data));
  });
});
