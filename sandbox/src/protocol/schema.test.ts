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
      { name: "chart", level: "strict", binding: "react", code: "export default () => null", libraries: { "@kit/d3": { place: "@kit/d3@7.9.5/dist/d3.js", code: new Blob(["export {}"]) } } },
      {
        name: "diagram",
        level: "strict",
        binding: "plain",
        code: new Blob(["export default () => undefined"]),
        place: "@kit/mermaid@1.0.0/dist/diagram.js",
        libraries: { "@kit/three": { place: "@kit/three@0.170.0/dist/three.js", code: new Blob(["export {}"]), libraries: { "@kit/math": { place: "@kit/math@1.0.0/dist/math.js", code: new Blob([]) } } } },
      },
      { name: "mermaid", level: "strict", module: { url: "/packs/m.js", integrity: "sha384-x" } },
      { name: "stars", level: "trusted" },
    ],
  },
  { type: "part", part: { node: { type: "componentLeaf", name: "map", params: [] }, inputs: {} } },
  { type: "styles", urls: ["/docs.css"] },
  { type: "scheme", scheme: "dark" },
  { type: "host" },
  { type: "ping", n: 1 },
  { type: "compile", name: "chart", language: "jsx", code: "export default () => null" },
  { type: "observe", on: true },
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
  {
    type: "observed",
    events: [
      { type: "frame", frame: "1", level: "permissive", renders: { kind: "use", component: "map" }, network: ["https://tiles.example.com"], at: 1 },
      { type: "frame", frame: "2", level: "strict", renders: { kind: "page" }, at: 1 },
      { type: "gone", frame: "1/1", why: "hung", at: 2 },
      { type: "message", frame: "1", way: "down", data: { type: "part", part: {} }, at: 3 },
      { type: "mounted", frame: "", use: 1, component: "legend", ms: 0.4, at: 4 },
      { type: "updated", frame: "", use: 1, ms: 0.1, at: 5 },
      { type: "unmounted", frame: "", use: 1, at: 6 },
      { type: "fallback", frame: "", component: "legend", reason: "threw", message: "boom", at: 7 },
      { type: "dropped", frame: "", count: 12, at: 8 },
    ],
  },
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

  const diagram = { name: "diagram", level: "strict", code: new Blob([]) };
  it.each([
    ["a frame's level can't be trusted", { type: "level", level: "trusted" }],
    ["a set has a value", { type: "set", input: "x" }],
    ["a message has only its own fields", { type: "ping", n: 1, extra: true }],
    ["a message is of a known type", { type: "other" }],
    ["a Blob module is plain", { type: "code", components: [{ ...diagram, binding: "react", place: "@kit/mermaid@1.0.0/dist/diagram.js" }] }],
    ["a Blob module has its place", { type: "code", components: [{ ...diagram, binding: "plain" }] }],
    ["a library has its place", { type: "code", components: [{ name: "chart", level: "strict", binding: "react", code: "", libraries: { "@kit/d3": { code: new Blob([]) } } }] }],
    ["a module by URL has no libraries", { type: "code", components: [{ name: "mermaid", level: "strict", module: { url: "/packs/m.js", integrity: "sha384-x" }, libraries: {} }] }],
    ["an observed frame's id is a path of numbers", { type: "observed", events: [{ type: "gone", frame: "../1", why: "hung", at: 1 }] }],
    ["an observed frame is of a frame's level", { type: "observed", events: [{ type: "frame", frame: "1", level: "trusted", renders: { kind: "page" }, at: 1 }] }],
    ["an observed event is of a known type", { type: "observed", events: [{ type: "other", frame: "", at: 1 }] }],
  ])("refuses what isn't a message: %s", (_rule, message) => {
    expect(validate(message), JSON.stringify(message)).toBe(false);
  });
});
