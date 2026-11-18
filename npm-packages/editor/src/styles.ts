// The stylesheets a host might give the page, which the playground lets the author try: `frame.css?host=<id>`.

export const HOST_STYLES = [
  { id: "reference", label: "Reference" },
  { id: "github", label: "GitHub-like" },
  { id: "docs", label: "Documentation" },
  { id: "fancy", label: "Fancy" },
  { id: "terminal", label: "Terminal" },
  { id: "magazine", label: "Magazine" },
  { id: "sketch", label: "Hand-drawn" },
  { id: "unstyled", label: "Unstyled" },
] as const;

export type HostStyle = (typeof HOST_STYLES)[number]["id"];

/** The host's stylesheet as the page's frames load it: one file, each stylesheet under its own link. */
export const stylesheetOf = (style: HostStyle): string[] => [new URL(`./frame.css?host=${style}`, location.href).href];
