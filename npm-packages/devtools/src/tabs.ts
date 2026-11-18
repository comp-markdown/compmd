// The tools' tabs (docs/devtools.md, decision 8): the editor has two, the page and the CompMD Inspector, which holds the
// views as tabs of its own. The inspector alone has only these, since the page is the host's own. The Console is none of
// them: it is a drawer under the Inspector's views, with the document's inputs beside it, as a browser's developer
// tools have it.

import type { ViewKind } from "./views.ts";

/** The editor's two tabs: the page, and the CompMD Inspector with the views. */
export type Section = "page" | "inspector";

/** The Inspector's tabs, in order: the first shows when it opens. */
export const TABS = [
  { id: "parse", label: "Parse tree" },
  { id: "expanded", label: "Expanded tree" },
  { id: "data", label: "Data" },
  { id: "components", label: "Components" },
  { id: "sandbox", label: "Sandbox" },
  { id: "messages", label: "Messages" },
  { id: "text", label: "Plain Text" },
  { id: "plain", label: "Plain Markdown" },
] as const;

export type Tab = (typeof TABS)[number]["id"];

/** The tabs the views' worker doesn't fill: the sandbox's observer fills two, and the text is the document's own. */
export type OwnTab = "sandbox" | "messages" | "text";

/** The tabs the views' worker fills, from the document as it is now. */
export type ViewTab = Exclude<Tab, OwnTab>;

export const isViewTab = (tab: Tab): tab is ViewTab => tab !== "sandbox" && tab !== "messages" && tab !== "text";

/** The view the views' worker makes for `tab`. */
export const kindOf = (tab: ViewTab): ViewKind => (tab === "parse" || tab === "expanded" ? "trees" : tab);
