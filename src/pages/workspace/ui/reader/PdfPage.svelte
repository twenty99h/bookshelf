<script lang="ts">
  import type { PDFDocumentProxy, RenderTask } from "pdfjs-dist";
  import type { SourceFragment } from "@/shared/api";
  import "./pdf-text-layer.css";

  type SavedSource = { draftId: string; fragment: SourceFragment };

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
    sources?: SavedSource[];
    onSourceSelect: (draftId: string, source: SourceFragment) => void;
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
      highlightSavedSources(
        targetTextLayer,
        sources.map((source) => source.fragment),
      );
    } catch (cause) {
      if (cause instanceof Error && cause.name !== "RenderingCancelledException") error = cause.message;
    }
  }

  function highlightSavedSources(textLayer: HTMLDivElement, savedSources: SourceFragment[]) {
    const spans = [...textLayer.querySelectorAll<HTMLElement>("span")];
    const segments: { span: HTMLElement; start: number; end: number }[] = [];
    let pageText = "";
    for (const span of spans) {
      const text = normalizeText(span.textContent ?? "");
      if (!text) continue;
      const start = pageText.length;
      pageText += `${pageText ? " " : ""}${text}`;
      const adjustedStart = start + (start ? 1 : 0);
      segments.push({ span, start: adjustedStart, end: adjustedStart + text.length });
    }
    for (const source of savedSources) {
      const excerpt = normalizeText(source.excerpt);
      const start = pageText.indexOf(excerpt);
      if (start < 0) continue;
      const end = start + excerpt.length;
      for (const segment of segments) {
        if (segment.start < end && segment.end > start) segment.span.dataset.sourceHighlight = "true";
      }
    }
  }

  function normalizeText(value: string) {
    return value.replace(/\s+/g, " ").trim().toLocaleLowerCase("ru");
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
      {#each sources as source, index (`${source.draftId}-${source.fragment.page}-${source.fragment.excerpt}`)}<button
          aria-label="Открыть сохранённый источник на странице {page}"
          class="absolute right-1 grid size-4 place-items-center rounded-full border border-amber bg-[#40331f] text-[9px] text-amber outline-offset-2 focus-visible:outline-2 focus-visible:outline-amber"
          style:top={`${18 + index * 7}%`}
          onclick={() => onSourceSelect(source.draftId, source.fragment)}>{index + 1}</button
        >{/each}
    </div>{/if}
  {#if error}<p class="absolute inset-x-8 top-8 rounded bg-white p-4 text-sm text-danger" role="alert">
      Не удалось отобразить страницу {page}: {error}
    </p>{/if}
</article>
