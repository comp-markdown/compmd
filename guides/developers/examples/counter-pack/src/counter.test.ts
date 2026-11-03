import { mountForTest } from "@compmd/pack/testing";
import { describe, expect, it } from "vitest";
import mount from "./counter.tsx";

describe("counter", () => {
  it("counts the reader's clicks from its start, and keeps the count when its label changes", async () => {
    const shown = mountForTest(mount, { params: { label: "Cups of tea", start: 2 }, fallback: "Cups of tea: 2" });
    const button = () => shown.target.querySelector("button");
    expect(button()?.textContent).toBe("Cups of tea: 2");
    button()?.click();
    await expect.poll(() => button()?.textContent).toBe("Cups of tea: 3");
    shown.update({ label: "Cups of coffee", start: 2 });
    expect(button()?.textContent).toBe("Cups of coffee: 3");
    shown.destroy();
    expect(shown.target.textContent).toBe("");
  });
});
