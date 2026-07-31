import { describe, expect, it } from "vitest";
import { extractPdfSelection } from "./pdf-selection";

describe("PDF selection source seam", () => {
  it("splits a cross-page range into the text selected on each page", () => {
    document.body.innerHTML = `
      <main id="reader">
        <section data-pdf-page="1"><span>Before </span><span id="start">first page ending</span></section>
        <section data-pdf-page="2"><span id="end">second page beginning</span><span> after</span></section>
      </main>`;
    const range = document.createRange();
    range.setStart(document.querySelector("#start")!.firstChild!, 0);
    range.setEnd(document.querySelector("#end")!.firstChild!, "second page".length);
    const selection = window.getSelection()!;
    selection.removeAllRanges();
    selection.addRange(range);

    expect(extractPdfSelection(document.querySelector("#reader")!, selection)).toEqual([
      { page: 1, excerpt: "first page ending", context: "Before first page ending" },
      { page: 2, excerpt: "second page", context: "second page beginning after" },
    ]);
  });
});
