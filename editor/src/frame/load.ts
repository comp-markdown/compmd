// A compiled component loaded as a module, whose imports the page's import map resolves, and mounted through its
// binding (spec §7.6): React's for JSX and TSX, Svelte's for Svelte, and a plain component's default export is its
// mount.

import type { HostComponent } from "@compmd/processor";
import type { Mount } from "@compmd/render";
import { react } from "@compmd/render/react";
import { svelte } from "@compmd/render/svelte";
import type { ComponentType } from "react";
import type { Component } from "svelte";
import { compile } from "./compile.ts";
import type { Loaded } from "./components.ts";

/** Loads `code` as a module. */
async function imported(code: string): Promise<{ default?: unknown }> {
  const url = URL.createObjectURL(new Blob([code], { type: "text/javascript" }));
  try {
    return (await import(/* @vite-ignore */ url)) as { default?: unknown };
  } finally {
    URL.revokeObjectURL(url);
  }
}

/**
 * `component` compiled, loaded and bound.
 *
 * @throws Error when it doesn't compile or load, or its module has no default export to mount.
 */
export async function load(component: HostComponent): Promise<Loaded> {
  const compiled = await compile(component);
  const exported = (await imported(compiled.code)).default;
  if (typeof exported !== "function") throw new Error(`\`${component.name}\`'s module has no default export to mount`);
  const css = compiled.css ? { css: compiled.css } : {};
  switch (component.language) {
    case "jsx":
    case "tsx":
      return { mount: react(exported as ComponentType<Record<string, unknown>>), ...css };
    case "svelte":
      return { mount: svelte(exported as Component<Record<string, unknown>>), ...css };
    case "js":
    case "ts":
      return { mount: exported as Mount, ...css };
  }
}
