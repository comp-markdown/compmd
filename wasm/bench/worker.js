// Times `process` on each document in this worker, as an editor's preview runs it: each run is a keystroke, the
// document's text changed and the whole request processed again, JSON in and out included.

import { init, process } from "../dist/index.js";

const WARM_UP = 20;
const RUNS = 200;

function percentile(times, q) {
  const sorted = [...times].sort((a, b) => a - b);
  return sorted[Math.round((sorted.length - 1) * q)];
}

const started = performance.now();
await init();
const startup = performance.now() - started;

const documents = await (await fetch("/documents.json")).json();
const results = documents.map(({ name, request }) => {
  const text = request.files[request.path];
  const times = [];
  for (let run = 0; run < WARM_UP + RUNS; run++) {
    request.files[request.path] = run % 2 === 0 ? text : `${text} `;
    const start = performance.now();
    process(request);
    if (run >= WARM_UP) times.push(performance.now() - start);
  }
  return { name, p50: percentile(times, 0.5), p95: percentile(times, 0.95) };
});

postMessage({ userAgent: navigator.userAgent, isolated: self.crossOriginIsolated, startup, results });
