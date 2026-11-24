/**
 * What `error` says, as text: an `Error`'s message, anything else as `String` writes it.
 *
 * @example
 * messageOf(new Error("no frame")); // "no frame"
 * messageOf(404); // "404"
 */
export const messageOf = (error: unknown): string => (error instanceof Error ? error.message : String(error));
