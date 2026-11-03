// The tools' tabs (docs/devtools.md, decision 8): the editor has two, the page and the CompMD Inspector, which holds the
// views as tabs of its own. The inspector alone has only these, since the page is the host's own. The Console is none of
// them: it is a drawer under whichever shows, as a browser's developer tools have it.

import type { ViewKind } from "./views.ts";

/** The editor's two tabs: the page, and the CompMD Inspector with the views. */
export type Section = "page" | "inspector";

/** The Inspector's tabs, in order. */
export const TABS = [
  { id: "plain", label: "Plain Markdown" },
  { id: "parse", label: "Parse tree" },
  { id: "expanded", label: "Expanded tree" },
  { id: "data", label: "Data" },
  { id: "components", label: "Components" },
  { id: "sandbox", label: "Sandbox" },
  { id: "messages", label: "Messages" },
] as const;

export type Tab = (typeof TABS)[number]["id"];

/** The tabs the sandbox's observer fills. */
export type ObservedTab = "sandbox" | "messages";

/** The tabs the views' worker fills, from the document as it is now. */
export type ViewTab = Exclude<Tab, ObservedTab>;

export const isViewTab = (tab: Tab): tab is ViewTab => tab !== "sandbox" && tab !== "messages";

/** The view the views' worker makes for `tab`. */
export const kindOf = (tab: ViewTab): ViewKind => (tab === "parse" || tab === "expanded" ? "trees" : tab);
