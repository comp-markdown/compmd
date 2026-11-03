// Work that follows a stream of events, done once a frame however many came.

/** `run`, called once in the next animation frame however often the returned function is called before it. */
export function onceAFrame(run: () => void): () => void {
  let waiting = false;
  return () => {
    if (waiting) return;
    waiting = true;
    requestAnimationFrame(() => {
      waiting = false;
      run();
    });
  };
}
