import { readFileSync } from "node:fs";
import { join } from "node:path";
import { Ajv2020 } from "ajv/dist/2020.js";
import { describe, expect, it } from "vitest";
import { FROM_FRAME, TO_FRAME, type FromFrame, type ToFrame } from "./messages.ts";

const schema = JSON.parse(readFileSync(join(import.meta.dirname, "../../protocol.schema.json"), "utf8")) as {
  $defs: Record<string, { oneOf?: { $ref: string }[]; properties?: { type?: { const?: string } } }>;
};
const validate = new Ajv2020({ strict: true }).compile(schema);

/** The message types a direction of the schema lists. */
const typesOf = (direction: "toFrame" | "fromFrame"): string[] =>
  (schema.$defs[direction]?.oneOf ?? []).map(({ $ref }) => schema.$defs[$ref.split("/").at(-1) ?? ""]?.properties?.type?.const ?? "");

/** One of each message, as the protocol's ends send them. */
const MESSAGES: (ToFrame | FromFrame)[] = [
  { type: "level", level: "permissive", policy: "connect-src https://tiles.example.com" },
  {
    type: "code",
    components: [
      { name: "chart", level: "strict", binding: "react", code: "export default () => null" },
      { name: "mermaid", level: "strict", module: { url: "/packs/m.js", integrity: "sha384-x" } },
      { name: "stars", level: "trusted" },
    ],
  },
  { type: "part", part: { node: { type: "componentLeaf", name: "map", params: [] }, inputs: {} } },
  { type: "styles", urls: ["/docs.css"] },
  { type: "host" },
  { type: "ping", n: 1 },
  { type: "compile", name: "chart", language: "jsx", code: "export default () => null" },
  { type: "ready", sandbox: 1 },
  { type: "loaded" },
  { type: "set", input: "region", value: "eu" },
  { type: "error", reason: "not-here", component: "map", message: "a permissive component can't be in a strict frame" },
  { type: "link", url: "https://example.com" },
  { type: "rendered", ms: 2.5 },
  { type: "size", width: 320, height: 200 },
  { type: "pong", n: 1 },
  { type: "compiled", name: "card", code: "", css: "p{}" },
  { type: "failed", name: "card", message: "unexpected token" },
];

describe("protocol.schema.json", () => {
  it("has every message type, each way", () => {
    expect(typesOf("toFrame")).toEqual(TO_FRAME);
    expect(typesOf("fromFrame")).toEqual(FROM_FRAME);
  });

  it("takes every message the protocol sends", () => {
    for (const message of MESSAGES) expect(validate(message), JSON.stringify(validate.errors)).toBe(true);
    expect(MESSAGES.map(({ type }) => type)).toEqual([...TO_FRAME, ...FROM_FRAME]);
  });

  it("refuses what isn't a message", () => {
    for (const message of [{ type: "level", level: "trusted" }, { type: "set", input: "x" }, { type: "ping", n: 1, extra: true }, { type: "other" }]) {
      expect(validate(message), JSON.stringify(message)).toBe(false);
    }
  });
});
