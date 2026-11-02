// The browser tests' registry, in Node before the tests: the local registry, whose URL the tests are given.

import type { TestProject } from "vitest/node";
import { localRegistry } from "./local-registry.ts";

declare module "vitest" {
  interface ProvidedContext {
    /** The local registry, ending in `/`. */
    registry: string;
  }
}

export default async function setup(project: TestProject): Promise<() => Promise<void>> {
  const registry = await localRegistry();
  project.provide("registry", registry.url);
  return () => registry.close();
}
