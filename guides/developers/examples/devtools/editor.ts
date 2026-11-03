import { mountDevtools } from "@compmd/devtools";
import { request, sandbox } from "virtual:compmd";
import text from "./notes.comp.md?raw";

mountDevtools(document.getElementById("tools") as HTMLElement, {
  sandbox,
  document: { request: { ...request, path: "notes.comp.md", files: { "notes.comp.md": text }, now: new Date().toISOString() } },
});
