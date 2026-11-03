// @compmd/devtools: the CompMD Developer Tools as a part any page embeds (docs/devtools.md). Written in Svelte, compiled
// into the package, so a page embeds it with a function, whatever it is written in; it renders in a shadow root, so the
// page's styles can't reach in, and its own can't leak out.

import type { SandboxOptions } from "@compmd/sandbox";
import { mount, unmount } from "svelte";
import Devtools from "./Devtools.svelte";
import css from "./devtools.css?inline";
import type { DevtoolsDocument } from "./document.ts";
import type { EditorState } from "./editor.ts";
import type { Registries } from "./protocol.ts";

export type { DevtoolsDocument, SupportingFile } from "./document.ts";
export type { EditorState } from "./editor.ts";
export type { Registries } from "./protocol.ts";
export type { Timings } from "./timings.ts";

export interface DevtoolsOptions {
  /**
   * The sandbox the page renders in, as `createSandbox` takes it (`@compmd/sandbox`): where its frame documents are
   * served, its built modules. The tools add the modules they fetch from a registry.
   */
  sandbox: SandboxOptions;
  /** The document to open first. */
  document: DevtoolsDocument;
  /** Where to fetch the packs a document imports that its request doesn't hold; without it, those are F026. */
  registries?: Registries;
  /** The host's stylesheets for the page, served from the frames' origin. */
  stylesheets?: readonly string[];
  /**
   * A passage of the text to select once the first document opens: a text fragment's `text=` value, its parts
   * percent-encoded as written (`?select=` in a link to the tools).
   */
  select?: string;
  /**
   * The link that opens this document with the passage `select` selected, the same value as above: given it, the tools
   * offer to copy a link to the selected text.
   */
  shareLink?: (select: string) => string;
}

/** The tools, mounted. */
export interface DevtoolsHandle {
  /** Opens `document` in place of the one open, in a new page. */
  open(document: DevtoolsDocument): void;
  /** The host's stylesheets for the page, in place of those it had. */
  styles(urls: readonly string[]): void;
  /** What the editor shows besides the page: its diagnostics, problems and timings; `undefined` until it opened. */
  readonly state: EditorState | undefined;
  destroy(): void;
}

/** What `Devtools.svelte` exports. */
interface ToolsExports {
  open(document: DevtoolsDocument): void;
  styles(urls: readonly string[]): void;
  editorState(): EditorState | undefined;
}

/**
 * Mounts the tools in `target`'s shadow root, which it makes, with `options.document` open.
 *
 * Its workers are files of their own next to it (`new Worker(new URL("./worker.js", import.meta.url))`), with the
 * processor's WebAssembly: a host's bundler that handles workers (Vite, webpack 5) builds them with the page. Vite builds
 * a package's workers only when it doesn't pre-bundle it: such a host lists `@compmd/devtools` in `optimizeDeps.exclude`.
 *
 * @example
 * const tools = mountDevtools(element, { sandbox: { base: "/sandbox/" }, document: { request } });
 * tools.open({ request: other });
 * tools.destroy();
 */
export function mountDevtools(target: HTMLElement, options: DevtoolsOptions): DevtoolsHandle {
  const root = target.shadowRoot ?? target.attachShadow({ mode: "open" });
  const sheet = new CSSStyleSheet();
  sheet.replaceSync(css);
  root.adoptedStyleSheets = [sheet];
  // What the component exports, typed here: only svelte-check reads a `.svelte` file's own types.
  const tools = mount(Devtools, { target: root, props: options }) as ToolsExports;
  return {
    open: (document) => {
      tools.open(document);
    },
    styles: (urls) => {
      tools.styles(urls);
    },
    get state() {
      return tools.editorState();
    },
    destroy: () => {
      void unmount(tools);
    },
  };
}
