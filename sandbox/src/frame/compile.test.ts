import { describe, expect, it } from "vitest";
import type { CompileRequest } from "../protocol/messages.ts";
import { bindingOf, compile } from "./compile.ts";

const component = (language: CompileRequest["language"], code: string, runtime?: string): CompileRequest => ({
  name: "card",
  language,
  code,
  ...(runtime ? { runtime } : {}),
});

describe("compile", () => {
  it("turns JSX into calls of React's JSX runtime, imports kept for the import map", async () => {
    const { code } = await compile(component("jsx", "import { useState } from 'react';\nexport default () => <b>{useState(1)[0]}</b>;", "react@19"));
    expect(code).toContain('from "react/jsx-runtime"');
    expect(code).toContain("from 'react'");
    expect(code).not.toContain("<b>");
  });

  it("strips TypeScript's types, with JSX in TSX", async () => {
    const ts = await compile(component("ts", "export default function mount(target: Element): void { target.textContent = 'x'; }"));
    expect(ts.code).not.toContain(": Element");
    const tsx = await compile(component("tsx", "export default ({ n }: { n: number }) => <i>{n}</i>;"));
    expect(tsx.code).toContain("react/jsx-runtime");
    expect(tsx.code).not.toContain("number }");
  });

  it("compiles Svelte for the client, its CSS apart", async () => {
    const compiled = await compile(component("svelte", "<script>let { title } = $props();</script>\n<h3>{title}</h3>\n<style>h3 { color: red; }</style>", "svelte@5"));
    expect(compiled.code).toContain("svelte/internal/client");
    expect(compiled.css).toMatch(/h3\.svelte-[\w-]+ \{/);
  });

  it("leaves plain JavaScript as it is", async () => {
    expect(await compile(component("js", "export default () => {};"))).toEqual({ code: "export default () => {};" });
  });

  it("compiles every component of the playground's charts", async () => {
    const files = import.meta.glob("../../../editor/examples/charts/*.{jsx,svelte}", { query: "?raw", import: "default", eager: true });
    const compiled = await Promise.all(
      Object.entries(files).map(([path, code]) => compile(component(path.endsWith(".svelte") ? "svelte" : "jsx", code))),
    );
    expect(compiled).toHaveLength(5);
    for (const { code } of compiled) expect(code).toMatch(/from ['"](react\/jsx-runtime|svelte\/internal\/client)['"]/);
  });

  it("fails on code that doesn't compile, and on a runtime the sandbox hasn't", async () => {
    await expect(compile(component("jsx", "export default () => <b>;"))).rejects.toThrow();
    await expect(compile(component("svelte", "{#if}"))).rejects.toThrow();
    await expect(compile(component("jsx", "x", "preact@10"))).rejects.toThrow("this sandbox has no preact@10 to run `card`");
  });

  it("names each language's binding", () => {
    expect([bindingOf("jsx"), bindingOf("tsx"), bindingOf("svelte"), bindingOf("js"), bindingOf("ts")]).toEqual(["react", "react", "svelte", "plain", "plain"]);
  });
});
