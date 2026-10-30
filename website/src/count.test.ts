import { Window } from "happy-dom";
import { describe, expect, it } from "vitest";
import { COUNTER, count, SCRIPT } from "./count.ts";

/** A document served from `url`. */
function documentAt(url: string): Document {
  return new Window({ url }).document as unknown as Document;
}

describe("count", () => {
  it("adds GoatCounter's script on the published site", () => {
    const document = documentAt("https://compmd.hakanserce.com/spec/");
    count(document);
    const script = document.head.querySelector("script");
    expect(script?.src).toBe(SCRIPT);
    expect(script?.dataset.goatcounter).toBe(COUNTER);
    expect(script?.async).toBe(true);
  });

  it("counts nothing served from this machine", () => {
    for (const url of ["http://localhost:5173/", "http://127.0.0.1:4173/", "http://[::1]:4191/"]) {
      const document = documentAt(url);
      count(document);
      expect(document.head.querySelector("script"), url).toBeNull();
    }
  });
});
