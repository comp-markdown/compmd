import * as svelte from "svelte/compiler";
import ts from "typescript";
import { describe, expect, it } from "vitest";
import { reactProps, svelteProps, type Compilers } from "./props.ts";

const compilers: Compilers = { ts, svelte };

describe("reactProps", () => {
  it("reads the names a default export destructures, and a rest", () => {
    expect(reactProps(ts, "export default function Badge({ label, tone = 'info', ...more }) { return null; }", "jsx")).toEqual({
      props: [{ name: "label" }, { name: "tone" }],
      rest: true,
    });
    expect(reactProps(ts, "export default () => null;", "jsx")).toEqual({ props: [], rest: false });
  });

  it("takes the types a type literal, an interface or an alias states", () => {
    const source = [
      "import type { ReactNode } from 'react';",
      "interface Props { title: string; count?: number; tone: 'info' | 'warn'; open: boolean | undefined; footer: ReactNode; meta: object }",
      "const Card = ({ title }: Props) => null;",
      "export default Card;",
    ].join("\n");
    expect(reactProps(ts, source, "tsx")).toEqual({
      props: [
        { name: "title", type: { kind: "string" } },
        { name: "count", type: { kind: "number" } },
        { name: "tone", type: { kind: "enum", members: ["info", "warn"] } },
        { name: "open", type: { kind: "boolean" } },
        { name: "footer", type: { kind: "slot" } },
        { name: "meta" },
      ],
      rest: false,
    });
    expect(reactProps(ts, "type P = { a: string } & { b: number };\nexport default function X(props: P) { return null; }", "tsx")?.props.map(({ name }) => name)).toEqual(["a", "b"]);
  });

  it("can't read what it doesn't see: no default export, a wrapped one, an untyped props object", () => {
    expect(reactProps(ts, "export function Badge() { return null; }", "jsx")).toBeUndefined();
    expect(reactProps(ts, "export default memo(function Badge({ a }) { return null; });", "jsx")).toBeUndefined();
    expect(reactProps(ts, "export default function Badge(props) { return null; }", "jsx")).toBeUndefined();
  });
});

describe("svelteProps", () => {
  it("reads what `$props()` is given, typed or not", () => {
    expect(svelteProps(compilers, "<script>\n  let { title, icon = 'info', children, footer } = $props();\n</script>\n<h3>{title}</h3>")).toEqual({
      props: [{ name: "title" }, { name: "icon" }, { name: "children" }, { name: "footer" }],
      rest: false,
    });
    const typed = "<script lang=\"ts\">\n  import type { Snippet } from 'svelte';\n  interface Props { value: number; children?: Snippet }\n  let { value, children }: Props = $props();\n</script>\n";
    expect(svelteProps(compilers, typed)).toEqual({ props: [{ name: "value", type: { kind: "number" } }, { name: "children", type: { kind: "slot" } }], rest: false });
  });

  it("gives no props for a component without `$props()`, and nothing without Svelte's compiler or for a source it can't parse", () => {
    expect(svelteProps(compilers, "<p>static</p>")).toEqual({ props: [], rest: false });
    expect(svelteProps(compilers, "<script>let x = 1;</script>")).toEqual({ props: [], rest: false });
    expect(svelteProps({ ts }, "<script>let { a } = $props();</script>")).toBeUndefined();
    expect(svelteProps(compilers, "<p>{</p>")).toBeUndefined();
  });
});
