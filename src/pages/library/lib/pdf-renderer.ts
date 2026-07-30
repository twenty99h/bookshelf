import type { PDFDocumentProxy } from "pdfjs-dist";

export async function renderPdfPage(
  pdfjs: typeof import("pdfjs-dist"),
  pdf: PDFDocumentProxy,
  pageNumber: number,
  zoom: number,
  canvas: HTMLCanvasElement,
  textLayer: HTMLDivElement,
): Promise<void> {
  const page = await pdf.getPage(pageNumber);
  const viewport = page.getViewport({ scale: zoom });
  const ratio = window.devicePixelRatio || 1;
  canvas.width = Math.floor(viewport.width * ratio);
  canvas.height = Math.floor(viewport.height * ratio);
  canvas.style.width = `${viewport.width}px`;
  canvas.style.height = `${viewport.height}px`;
  textLayer.replaceChildren();
  textLayer.style.width = `${viewport.width}px`;
  textLayer.style.height = `${viewport.height}px`;
  await page.render({ canvas, viewport, transform: ratio === 1 ? undefined : [ratio, 0, 0, ratio, 0, 0] }).promise;
  await new pdfjs.TextLayer({
    textContentSource: await page.getTextContent(),
    container: textLayer,
    viewport,
  }).render();
}

export function selectedPdfText(textLayer: HTMLDivElement): { excerpt: string; context: string } | null {
  const selection = window.getSelection();
  const excerpt = selection?.toString().trim() ?? "";
  if (!excerpt || !selection?.anchorNode || !textLayer.contains(selection.anchorNode)) return null;
  const text = textLayer.textContent?.replace(/\s+/g, " ").trim() ?? "";
  const start = Math.max(0, text.indexOf(excerpt) - 180);
  return { excerpt, context: text.slice(start, text.indexOf(excerpt) + excerpt.length + 180) };
}

export async function destroyPdf(pdf: PDFDocumentProxy | null, textLayer: HTMLDivElement | undefined): Promise<void> {
  textLayer?.replaceChildren();
  await pdf?.cleanup();
}
