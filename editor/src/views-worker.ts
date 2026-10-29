// The views' worker: makes the document's trees and its plain rendering, off the text area's thread.

import { init } from "@compmd/processor";
import { viewOf, type ViewRequest } from "./views.ts";

const ready = init();

addEventListener("message", ({ data }: MessageEvent<ViewRequest>) => {
  void ready.then(() => {
    postMessage(viewOf(data));
  });
});
