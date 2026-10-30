// A component's code loaded as a module, whose imports the frame's import map resolves, and mounted through its
// binding (spec §7.6): React's for JSX and TSX, Svelte's for Svelte, and a plain component's default export is its
// mount. Compiled text loads as a `blob:` module; a built module by its URL, checked against its file's hash.

import type { Mount } from "@compmd/render";
import { react } from "@compmd/render/react";
import { svelte } from "@compmd/render/svelte";
import type { ComponentType } from "react";
import type { Component } from "svelte";
import type { Runnable } from "../protocol/messages.ts";
import type { Loaded } from "./components.ts";

type Module = { default?: unknown };

/** Loads `code` as a module. */
async function imported(code: string): Promise<Module> {
  const url = URL.createObjectURL(new Blob([code], { type: "text/javascript" }));
  try {
    return (await import(/* @vite-ignore */ url)) as Module;
  } finally {
    URL.revokeObjectURL(url);
  }
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

/**
 * `component`'s code loaded and bound.
 *
 * @throws Error when it doesn't load, or its module has no default export to mount.
 */
export async function load(component: Runnable): Promise<Loaded> {
  const module = "module" in component ? await checked(component.module) : await imported(component.code);
  const exported = module.default;
  if (typeof exported !== "function") throw new Error(`\`${component.name}\`'s module has no default export to mount`);
  if ("module" in component) return { mount: exported as Mount };
  const css = component.css ? { css: component.css } : {};
  switch (component.binding) {
    case "react":
      return { mount: react(exported as ComponentType<Record<string, unknown>>), ...css };
    case "svelte":
      return { mount: svelte(exported as Component<Record<string, unknown>>), ...css };
    case "plain":
      return { mount: exported as Mount, ...css };
  }
}
