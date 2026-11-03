// The tools' tabs (docs/devtools.md, decision 8): the page first, then why something is wrong, then what the document
// is made of, then the raw material. The inspector has them all but the page, which is the host's own.

import type { ViewKind } from "./views.ts";

export const TABS = [
  { id: "page", label: "Page" },
  { id: "console", label: "Console" },
  { id: "components", label: "Components" },
  { id: "sandbox", label: "Sandbox" },
  { id: "data", label: "Data" },
  { id: "plain", label: "Plain Markdown" },
  { id: "messages", label: "Messages" },
  { id: "parse", label: "Parse tree" },
  { id: "expanded", label: "Expanded tree" },
] as const;

export type Tab = (typeof TABS)[number]["id"];

/** The tabs the sandbox's observer fills. */
export type ObservedTab = "console" | "sandbox" | "messages";

/** The tabs the views' worker fills, from the document as it is now. */
export type ViewTab = Exclude<Tab, "page" | ObservedTab>;

export const isViewTab = (tab: Tab): tab is ViewTab => tab !== "page" && tab !== "console" && tab !== "sandbox" && tab !== "messages";

/** The view the views' worker makes for `tab`. */
export const kindOf = (tab: ViewTab): ViewKind => (tab === "parse" || tab === "expanded" ? "trees" : tab);

/** The inspector's tabs: all but the page. */
export const INSPECTOR_TABS: readonly Tab[] = TABS.map(({ id }) => id).filter((id) => id !== "page");
