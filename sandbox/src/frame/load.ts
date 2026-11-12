// A component's code loaded as a module, whose imports the frame's import map resolves, and mounted through its
// binding (spec §7.6): React's for JSX and TSX, Svelte's for Svelte, and a plain component's default export is its
// mount. Compiled text and a built module the host's page holds load as `blob:` modules, their imports of library
// packs written as the URLs of the libraries' own `blob:` modules (docs/sharing-library-packs.md); a built module, or
// a pack's source component the host compiled ahead, by its URL, checked against its file's hash.

import type { Mount } from "@compmd/render";
import { react } from "@compmd/render/react";
import { svelte } from "@compmd/render/svelte";
import type { ComponentType } from "react";
import type { Component } from "svelte";
import type { Libraries, LibraryCode, Runnable } from "../protocol/messages.ts";
import type { Loaded } from "./components.ts";
import { withImports } from "./imports.ts";

type Module = { default?: unknown; css?: unknown };

const blobOf = (code: string | Blob): Blob => new Blob([code], { type: "text/javascript" });

/** Loads `code` as a module: a `Blob` keeps its bytes, given the type a module needs. */
async function imported(code: string | Blob): Promise<Module> {
  const url = URL.createObjectURL(blobOf(code));
  try {
    return (await import(/* @vite-ignore */ url)) as Module;
  } finally {
    URL.revokeObjectURL(url);
  }
}

/**
 * Each library's module URL, by its place: made once, the libraries it imports first, and kept for the frame's life,
 * so every component that imports it shares one instance.
 */
const libraryUrls = new Map<string, Promise<string>>();

function libraryUrl({ place, code, libraries }: LibraryCode): Promise<string> {
  let url = libraryUrls.get(place);
  if (!url) {
    url = (async () => URL.createObjectURL(blobOf(await written(await code.text(), libraries))))();
    libraryUrls.set(place, url);
  }
  return url;
}

/** `text` with its imports of `libraries` written as their URLs. */
async function written(text: string, libraries: Libraries | undefined): Promise<string> {
  const entries = Object.entries(libraries ?? {});
  if (entries.length === 0) return text;
  const urls = await Promise.all(entries.map(async ([name, library]) => [name, await libraryUrl(library)] as const));
  return withImports(text, Object.fromEntries(urls));
}

/** Loads code given as text or a `Blob`, with its library packs. */
async function importedWith(code: string | Blob, libraries: Libraries | undefined): Promise<Module> {
  if (!libraries || Object.keys(libraries).length === 0) return imported(code);
  return imported(await written(typeof code === "string" ? code : await code.text(), libraries));
}

/**
 * Loads the module at `url` with a script that has `integrity`, so the browser runs it only if its file matches, then
 * imports it: the module the script loaded, since a page loads a module once per URL.
 */
async function checked(module: { url: string; integrity: string }): Promise<Module> {
  const url = new URL(module.url, document.baseURI).href;
  await new Promise<void>((resolve, reject) => {
    const script = document.createElement("script");
    script.type = "module";
    script.src = url;
    script.integrity = module.integrity;
    script.addEventListener("load", () => {
      resolve();
    });
    script.addEventListener("error", () => {
      reject(new Error(`\`${module.url}\` didn't load, or doesn't match its hash`));
    });
    document.head.append(script);
  });
  return (await import(/* @vite-ignore */ url)) as Module;
}

/** A component's styles: the compiled text's, or the `css` export of a module compiled ahead; none when empty. */
function cssOf(component: Runnable, module: Module): { css?: string } {
  const css = "module" in component ? module.css : "css" in component ? component.css : undefined;
  return typeof css === "string" && css !== "" ? { css } : {};
}

/**
 * `component`'s code loaded and bound.
 *
 * @throws Error when it doesn't load, or its module has no default export to mount.
 */
export async function load(component: Runnable): Promise<Loaded> {
  const module = "module" in component ? await checked(component.module) : await importedWith(component.code, component.libraries);
  const exported = module.default;
  if (typeof exported !== "function") throw new Error(`\`${component.name}\`'s module has no default export to mount`);
  // A built module is a plain mount; one compiled ahead mounts through its binding, its styles in its `css` export.
  const binding = component.binding ?? "plain";
  if (binding === "plain" && "module" in component) return { mount: exported as Mount };
  const css = cssOf(component, module);
  switch (binding) {
    case "react":
      return { mount: react(exported as ComponentType<Record<string, unknown>>), ...css };
    case "svelte":
      return { mount: svelte(exported as Component<Record<string, unknown>>), ...css };
    case "plain":
      return { mount: exported as Mount, ...css };
  }
}
