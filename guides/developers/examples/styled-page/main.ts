import { init, OpenDocument, plainHtml } from "@compmd/processor";
import { createSandbox } from "@compmd/sandbox";
import { request, sandbox } from "virtual:compmd";
import text from "./welcome.comp.md?raw";

await init();

const opened = new OpenDocument({ ...request, path: "welcome.comp.md", files: { "welcome.comp.md": text }, now: new Date().toISOString() });
const view = createSandbox(sandbox).open(document.getElementById("page") as HTMLElement, { components: opened.components });

// The app's look: its stylesheet in the page's frame, and the theme tokens the built-ins' styles read.
view.styles([new URL("page.css", location.href).href]);
view.theme({ "--compmd-accent": "#0b7285", "--compmd-space-m": "1.25rem", "--compmd-radius-m": "12px" });
view.render(opened.expanded);

// Light or dark is the page's `color-scheme`; after the app switches it, the view reads it again.
const scheme = document.getElementById("scheme") as HTMLButtonElement;
scheme.addEventListener("click", () => {
  const dark = document.documentElement.style.colorScheme !== "dark";
  document.documentElement.style.colorScheme = dark ? "dark" : "light";
  scheme.textContent = dark ? "Light" : "Dark";
  view.scheme();
});

// A link the reader clicked in the page: show where it goes, and open it only when the reader says so.
const dialog = document.getElementById("link") as HTMLDialogElement;
view.on("link", ({ url }) => {
  const target = URL.parse(url, location.href);
  if (!target || !["https:", "http:", "mailto:"].includes(target.protocol)) return;
  (dialog.querySelector("output") as HTMLOutputElement).value = target.href;
  dialog.onclose = () => {
    if (dialog.returnValue === "open") window.open(target.href, "_blank", "noopener");
  };
  dialog.showModal();
});

// What a plain Markdown renderer shows of the same text, in a frame that runs nothing.
(document.getElementById("plain") as HTMLIFrameElement).srcdoc = plainHtml(text);
