// Times the processor in this worker, as an editor's preview runs it, JSON in and out included:
// - `process` on each document: each run is a keystroke, the document's text changed and the whole request processed
//   again;
// - a keystroke in an open document: a character typed into a paragraph in the middle of the body, then deleted, in
//   turn, timed in the module alone (the processing, and the splice as JSON text) and through `OpenDocument`, which
//   also parses and applies the splice.

import { init, OpenDocument, process } from "../dist/index.js";
import { Document } from "../pkg/compmd_wasm.js";

const WARM_UP = 20;
const RUNS = 200;

function percentile(times, q) {
  const sorted = [...times].sort((a, b) => a - b);
  return sorted[Math.round((sorted.length - 1) * q)];
}

/** `run(i)` timed `RUNS` times after `WARM_UP` runs: its p50 and p95. */
function timed(run) {
  const times = [];
  for (let i = 0; i < WARM_UP + RUNS; i++) {
    const start = performance.now();
    run(i);
    if (i >= WARM_UP) times.push(performance.now() - start);
  }
  return { p50: percentile(times, 0.5), p95: percentile(times, 0.95) };
}

const started = performance.now();
await init();
const startup = performance.now() - started;

const { documents, keystrokes } = await (await fetch("/documents.json")).json();
const results = documents.map(({ name, request }) => {
  const text = request.files[request.path];
  return {
    name,
    ...timed((run) => {
      request.files[request.path] = run % 2 === 0 ? text : `${text} `;
      process(request);
    }),
  };
});

for (const { name, request } of keystrokes) {
  const text = request.files[request.path];
  // Inside the sentence the middle of the body is in.
  const at = text.indexOf(".", text.length / 2);
  const keystroke = (edit) => (run) => (run % 2 === 0 ? edit(at, at, "x") : edit(at, at + 1, ""));
  const kept = new Document(JSON.stringify(request));
  results.push({
    name: `${name}, processing`,
    ...timed(
      keystroke((start, end, inserted) => {
        kept.edit(start, end, inserted);
        kept.splice();
      }),
    ),
  });
  kept.free();
  const open = new OpenDocument(request);
  results.push({ name: `${name}, with the splice applied`, ...timed(keystroke((start, end, inserted) => open.edit(start, end, inserted))) });
  open.free();
}

postMessage({ userAgent: navigator.userAgent, isolated: self.crossOriginIsolated, startup, results });
