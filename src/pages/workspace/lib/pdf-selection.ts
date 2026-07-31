import type { SourceFragment } from "@/shared/api";

export function extractPdfSelection(container: HTMLElement, selection: Selection): SourceFragment[] {
  if (
    selection.isCollapsed ||
    selection.rangeCount === 0 ||
    !selection.anchorNode ||
    !container.contains(selection.anchorNode)
  ) {
    return [];
  }
  const selectedRange = selection.getRangeAt(0);
  return [...container.querySelectorAll<HTMLElement>("[data-pdf-page]")]
    .filter((page) => selectedRange.intersectsNode(page))
    .map((page) => {
      const pageRange = document.createRange();
      pageRange.selectNodeContents(page);
      const fragmentRange = selectedRange.cloneRange();
      if (fragmentRange.compareBoundaryPoints(Range.START_TO_START, pageRange) < 0) {
        fragmentRange.setStart(pageRange.startContainer, pageRange.startOffset);
      }
      if (fragmentRange.compareBoundaryPoints(Range.END_TO_END, pageRange) > 0) {
        fragmentRange.setEnd(pageRange.endContainer, pageRange.endOffset);
      }
      return {
        page: Number(page.dataset.pdfPage),
        excerpt: fragmentRange.toString().replace(/\s+/g, " ").trim(),
        context: page.textContent?.replace(/\s+/g, " ").trim().slice(0, 500) ?? "",
      };
    })
    .filter((fragment) => fragment.page > 0 && fragment.excerpt.length > 0);
}
