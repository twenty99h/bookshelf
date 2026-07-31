<script lang="ts">
  import type { PDFDocumentProxy, RenderTask } from "pdfjs-dist";
  import type { SourceFragment } from "@/shared/api";
  import "./pdf-text-layer.css";

  let {
    document,
    page,
    zoom,
    mode,
    invertImages,
    sources = [],
    onSourceSelect,
  }: {
    document: PDFDocumentProxy;
    page: number;
    zoom: number;
    mode: "muted" | "original" | "dark";
    invertImages: boolean;
    sources?: SourceFragment[];
    onSourceSelect: (source: SourceFragment) => void;
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
  {#if sources.length}<div
      class="absolute inset-y-0 right-0 w-6 border-l border-amber/40"
      aria-label="Цифровое поле источников"
    >
      {#each sources as source, index (`${source.page}-${source.excerpt}`)}<button
          aria-label="Открыть сохранённый источник на странице {page}"
          class="absolute right-1 grid size-4 place-items-center rounded-full border border-amber bg-[#40331f] text-[9px] text-amber outline-offset-2 focus-visible:outline-2 focus-visible:outline-amber"
          style:top={`${18 + index * 7}%`}
          onclick={() => onSourceSelect(source)}>{index + 1}</button
        >{/each}
    </div>{/if}
  {#if error}<p class="absolute inset-x-8 top-8 rounded bg-white p-4 text-sm text-danger" role="alert">
      Не удалось отобразить страницу {page}: {error}
    </p>{/if}
</article>
