// Time a frame is given, counted only while this window's thread answers. A page and its frames may share one thread
// (WebKit, and Chromium's frames in the page's process): while it is busy, with another frame's module say, a frame
// can neither load nor answer, and a clock's time would count against it. A tick that comes late counts as one tick.

/** How often the awake time is counted, in ms: what a busy stretch of any length counts as, at most. */
export const AWAKE_TICK_MS = 250;

/**
 * Calls `done` once this thread has been awake for `ms`, counted in ticks of `AWAKE_TICK_MS`: a stretch the thread was
 * busy for counts as one tick, however long. Returns what cancels it.
 *
 * @example
 * const cancel = afterAwake(5000, () => stop({ reason: "unloaded" }));
 * port.addEventListener("message", cancel);
 */
export function afterAwake(ms: number, done: () => void): () => void {
  let awake = 0;
  const ticks = setInterval(() => {
    awake += AWAKE_TICK_MS;
    if (awake < ms) return;
    clearInterval(ticks);
    done();
  }, AWAKE_TICK_MS);
  return () => {
    clearInterval(ticks);
  };
}
