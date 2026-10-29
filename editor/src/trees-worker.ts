// The trees' worker: processes a document from scratch for its parse and expanded trees, off the text area's thread.

import { init } from "@compmd/processor";
import { treesOf, type TreesRequest } from "./trees.ts";

const ready = init();

addEventListener("message", ({ data }: MessageEvent<TreesRequest>) => {
  void ready.then(() => {
    postMessage(treesOf(data));
  });
});
