import { expect, it } from "vitest";

it("runs in Node", () => {
  expect(typeof document).toBe("undefined");
});
