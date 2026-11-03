import { mountForTest } from "@compmd/pack/testing";
import { describe, expect, it } from "vitest";
import mount from "./lanes.ts";

/** A template slot's instance, as the renderer gives it: a record, and an element for what its template rendered. */
const instance = (title: string, status: string) => ({ value: { title, status }, items: [{ element: Object.assign(document.createElement("p"), { textContent: title }) }] });

describe("lanes", () => {
  it("puts each record's rendered template in the lane of its field, lanes in the order they first appear", () => {
    const rows = [
      { title: "Write", status: "doing" },
      { title: "Plan", status: "done" },
      { title: "Review", status: "doing" },
    ];
    const slots = { default: rows.map(({ title, status }) => instance(title, status)) };
    const shown = mountForTest(mount, { params: { rows, by: "status", as: "t" }, slots });
    const lanes = [...shown.target.querySelectorAll("section")].map((lane) => lane.textContent);
    expect(lanes).toEqual(["doingWriteReview", "donePlan"]);
    shown.destroy();
  });
});
