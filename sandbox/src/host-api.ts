// `host` (spec §7.4): a permissive component given `host` calls the host's functions by name and hears its events,
// over the port its frame got; the host's page answers with the object it gave the sandbox. A trusted component, in
// the host's page, gets the same calls and events from that object directly, so one component works at either level.

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

/** A function of `api` by `name`, its own, never one every object has (`toString`). */
const functionOf = (api: HostApi, name: string) => (Object.hasOwn(api, name) ? api[name] : undefined);

/** What `api` doesn't have, as a call to it fails. */
const missing = (name: string) => `the host has no \`${name}\``;

/** Listeners by event name, which `on` adds to and an event goes to. */
class Listeners {
  readonly #byName = new Map<string, Set<(data: unknown) => void>>();

  on(name: string, listener: (data: unknown) => void): () => void {
    const set = this.#byName.get(name) ?? new Set();
    set.add(listener);
    this.#byName.set(name, set);
    return () => set.delete(listener);
  }

  tell(name: string, data: unknown): void {
    for (const listener of this.#byName.get(name) ?? []) listener(data);
  }
}

/** `host` for a component in the frame that got `port`. */
export function hostClient(port: MessagePort): HostClient {
  let next = 0;
  const pending = new Map<number, { resolve(value: unknown): void; reject(error: Error): void }>();
  const listeners = new Listeners();
  port.addEventListener("message", ({ data }: MessageEvent<Answer | Event>) => {
    if ("event" in data) {
      listeners.tell(data.event, data.data);
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
    on: (name, listener) => listeners.on(name, listener),
  };
}

/**
 * `host` for the trusted components in the host's own page, answered by `api` directly, as `serveHost` answers a frame:
 * the same calls, failing alike, and the same events, which `emit` sends.
 *
 * @example
 * const { client, emit } = localHost({ rating: (product) => ratings.get(product) });
 * createPage(container, { components, host: client });
 * emit("changed", { product: "p-1" });
 */
export function localHost(api: HostApi): { client: HostClient; emit(event: string, data: unknown): void } {
  const listeners = new Listeners();
  return {
    client: {
      call: (name, ...args) => {
        const fn = functionOf(api, name);
        if (!fn) return Promise.reject(new Error(missing(name)));
        // A function that throws rejects, as across a frame, with what it threw as the message.
        return Promise.resolve()
          .then(() => fn(...args))
          .catch((error: unknown) => {
            throw error instanceof Error ? error : new Error(String(error));
          });
      },
      on: (name, listener) => listeners.on(name, listener),
    },
    emit: (event, data) => {
      listeners.tell(event, data);
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
    const fn = functionOf(api, data.name);
    if (!fn) {
      answer({ id: data.id, error: missing(data.name) });
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
