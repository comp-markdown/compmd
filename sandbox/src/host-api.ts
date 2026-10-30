// `host` across a frame (spec §7.4): a permissive component given `host` calls the host's functions by name and hears
// its events, over the port its frame got; the host's page answers with the object it gave the sandbox.

/** What a component in a frame gets as `host`. */
export interface HostClient {
  /** Calls the host's function `name` with `args`, which must be copyable; resolves with what it returned. */
  call(name: string, ...args: unknown[]): Promise<unknown>;
  /** Listens to the host's event `name`; returns a function that stops listening. */
  on(name: string, listener: (data: unknown) => void): () => void;
}

/** The host's side: its functions by name, and its events, which it sends with `emit`. */
export type HostApi = Readonly<Record<string, (...args: unknown[]) => unknown>>;

type Call = { id: number; name: string; args: unknown[] };
type Answer = { id: number; result: unknown } | { id: number; error: string };
type Event = { event: string; data: unknown };

/** `host` for a component in the frame that got `port`. */
export function hostClient(port: MessagePort): HostClient {
  let next = 0;
  const pending = new Map<number, { resolve(value: unknown): void; reject(error: Error): void }>();
  const listeners = new Map<string, Set<(data: unknown) => void>>();
  port.addEventListener("message", ({ data }: MessageEvent<Answer | Event>) => {
    if ("event" in data) {
      for (const listener of listeners.get(data.event) ?? []) listener(data.data);
      return;
    }
    const call = pending.get(data.id);
    pending.delete(data.id);
    if ("error" in data) call?.reject(new Error(data.error));
    else call?.resolve(data.result);
  });
  port.start();
  return {
    call: (name, ...args) =>
      new Promise((resolve, reject) => {
        next += 1;
        pending.set(next, { resolve, reject });
        port.postMessage({ id: next, name, args } satisfies Call);
      }),
    on: (name, listener) => {
      const set = listeners.get(name) ?? new Set();
      set.add(listener);
      listeners.set(name, set);
      return () => set.delete(listener);
    },
  };
}

/**
 * Answers the calls of the frame on the other end of `port` with `api`, and returns a function that sends it an
 * event. A name `api` doesn't have, or a function that throws, answers with an error.
 */
export function serveHost(port: MessagePort, api: HostApi): (event: string, data: unknown) => void {
  port.addEventListener("message", ({ data }: MessageEvent<Call>) => {
    const answer = (message: Answer) => {
      port.postMessage(message);
    };
    const fn = Object.hasOwn(api, data.name) ? api[data.name] : undefined;
    if (!fn) {
      answer({ id: data.id, error: `the host has no \`${data.name}\`` });
      return;
    }
    Promise.resolve()
      .then(() => fn(...data.args))
      .then(
        (result) => {
          answer({ id: data.id, result });
        },
        (error: unknown) => {
          answer({ id: data.id, error: error instanceof Error ? error.message : String(error) });
        },
      );
  });
  port.start();
  return (event, data) => {
    port.postMessage({ event, data } satisfies Event);
  };
}
