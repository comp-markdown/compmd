// The editor's worker: processing never runs in the text area's input handler, so each key press is painted first.

import { init, OpenDocumentJson } from "@compmd/processor";
import { Processing } from "./processing.ts";
import type { ToWorker } from "./protocol.ts";

const ready = init();
const processing = new Processing((request) => new OpenDocumentJson(request));

addEventListener("message", ({ data }: MessageEvent<ToWorker>) => {
  // Messages wait for the module in the order they came.
  void ready.then(() => {
    postMessage(processing.handle(data));
  });
});
