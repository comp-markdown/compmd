// A permissive frame's network: its parent sends the directives for its pack's origins (`networkPolicy`), and the
// frame adds them as its second policy (`narrowed`), with everything else its first policy allows and nothing more.

/** An origin as a policy names it: a scheme and a host, with nothing a policy would read as more. */
const ORIGIN = /^(https?|wss?):\/\/[^\s;,']+$/;

/**
 * The directives that give a permissive frame `origins`, for its parent to send in `level`.
 *
 * @throws Error for an entry that isn't an origin (`https://tiles.example.com`).
 */
export function networkPolicy(origins: readonly string[]): string {
  const wrong = origins.find((origin) => !ORIGIN.test(origin));
  if (wrong !== undefined) throw new Error(`\`${wrong}\` isn't an origin a permissive pack may reach`);
  const listed = origins.join(" ");
  const or = (sources: string) => [sources, listed].filter(Boolean).join(" ") || "'none'";
  return [`connect-src ${or("")}`, `img-src ${or("data: blob:")}`, `font-src ${or("'self'")}`, `media-src ${or("")}`].join("; ");
}

/**
 * The second policy of a permissive frame whose first is `first`: nothing but what `first` allows, `network` adds and
 * nested frames of `strictUrl`, its `strict.html`, since `default-src 'none'` covers every other kind of request.
 */
export function narrowed(first: string, network: string, strictUrl: string): string {
  return ["default-src 'none'", first, network, `frame-src ${strictUrl}`, `child-src ${strictUrl}`].join("; ");
}
