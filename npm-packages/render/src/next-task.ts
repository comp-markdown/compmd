/**
 * Resolves a task later, after the browser has had its turn: through a message, which no timer clamps, or a timer
 * where there is no `MessageChannel`.
 *
 * @example
 * for (const block of blocks) {
 *   await nextTask();
 *   highlight(block);
 * }
 */
export function nextTask(): Promise<void> {
  return new Promise((resolve) => {
    if (typeof MessageChannel !== "function") {
      setTimeout(resolve, 0);
      return;
    }
    const { port1, port2 } = new MessageChannel();
    port1.onmessage = () => {
      port1.close();
      resolve();
    };
    port2.postMessage(null);
  });
}
