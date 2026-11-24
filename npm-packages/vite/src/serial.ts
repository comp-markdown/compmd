// A rebuild on save, one at a time (audit B14, B20): `compmd-pack dev`'s modules and the Vite plugin's packs are built
// again when a file changes, and two quick saves must not build at once into the same files, nor a failed build end the
// dev server.

/** How a serialized job runs. */
export interface SerializedOptions {
  /** How long, in ms, the job waits after the last call before it runs: saves in a burst are one run. */
  wait?: number;
  /** Hears why a run failed; the next call runs it again. */
  onError: (error: unknown) => void;
}

/**
 * `job` as a function to call whenever it should run again: never two runs at once, calls in a burst one run, and a
 * call made while it runs one more run after it, with what changed meanwhile. A run that fails goes to `onError`, and
 * the job runs again at the next call.
 *
 * @example
 * const rebuild = serialized(() => buildModules(pack), { onError: (error) => console.error(error) });
 * watcher.on("change", rebuild);
 */
export function serialized(job: () => Promise<void>, { wait = 50, onError }: SerializedOptions): () => void {
  let timer: ReturnType<typeof setTimeout> | undefined;
  let running = false;
  let again = false;
  const run = async (): Promise<void> => {
    running = true;
    try {
      await job();
    } catch (error) {
      onError(error);
    } finally {
      running = false;
      if (again) {
        again = false;
        schedule();
      }
    }
  };
  const schedule = (): void => {
    clearTimeout(timer);
    timer = setTimeout(() => {
      if (running) again = true;
      else void run();
    }, wait);
  };
  return schedule;
}
