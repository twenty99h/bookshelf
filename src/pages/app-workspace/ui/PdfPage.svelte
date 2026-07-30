<script lang="ts">
  import type { PDFDocumentProxy, RenderTask } from "pdfjs-dist";
  import "pdfjs-dist/web/pdf_viewer.css";

  let {
    document,
    page,
    zoom,
    mode,
    invertImages,
  }: {
    document: PDFDocumentProxy;
    page: number;
    zoom: number;
    mode: "muted" | "original" | "dark";
    invertImages: boolean;
  } = $props();

  let renderTask: RenderTask | null = null;
  let error = $state("");

  function renderPage(node: HTMLElement) {
    const pdf = document;
    const pageNumber = page;
    const scale = zoom;
    const canvas = node.querySelector("canvas");
    const textLayer = node.querySelector<HTMLDivElement>(".textLayer");
    if (!canvas || !textLayer) return;
    void render(pdf, pageNumber, scale, canvas, textLayer);
    return () => renderTask?.cancel();
  }

  async function render(
    pdf: PDFDocumentProxy,
    pageNumber: number,
    scale: number,
    targetCanvas: HTMLCanvasElement,
    targetTextLayer: HTMLDivElement,
  ) {
    try {
      error = "";
      const pdfjs = await import("pdfjs-dist");
      const pdfPage = await pdf.getPage(pageNumber);
      const viewport = pdfPage.getViewport({ scale });
      const ratio = window.devicePixelRatio || 1;
      targetCanvas.width = Math.floor(viewport.width * ratio);
      targetCanvas.height = Math.floor(viewport.height * ratio);
      targetCanvas.style.width = `${viewport.width}px`;
      targetCanvas.style.height = `${viewport.height}px`;
      targetTextLayer.replaceChildren();
      targetTextLayer.style.width = `${viewport.width}px`;
      targetTextLayer.style.height = `${viewport.height}px`;
      renderTask = pdfPage.render({
        canvas: targetCanvas,
        viewport,
        transform: ratio === 1 ? undefined : [ratio, 0, 0, ratio, 0, 0],
      });
      await renderTask.promise;
      await new pdfjs.TextLayer({
        textContentSource: await pdfPage.getTextContent(),
        container: targetTextLayer,
        viewport,
      }).render();
    } catch (cause) {
      if (cause instanceof Error && cause.name !== "RenderingCancelledException") error = cause.message;
    }
  }
</script>

<article
  class="pdf-page relative mx-auto w-max overflow-hidden border border-black/20 shadow-[0_16px_45px_rgba(0,0,0,.28)]"
  class:bg-[#e9e7e0]={mode === "muted"}
  class:bg-white={mode === "original"}
  class:bg-[#d7d9db]={mode === "dark"}
  class:invert={mode === "dark"}
  class:[&_canvas]:invert={mode === "dark" && !invertImages}
  data-pdf-page={page}
  aria-label="Страница {page}"
  {@attach renderPage}
>
  <canvas></canvas>
  <div class="textLayer"></div>
  {#if error}<p class="absolute inset-x-8 top-8 rounded bg-white p-4 text-sm text-danger" role="alert">
      Не удалось отобразить страницу {page}: {error}
    </p>{/if}
</article>

<style>
  .pdf-page :global(.textLayer) {
    position: absolute;
    inset: 0;
  }
</style>
