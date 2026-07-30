<script lang="ts">
  import { onMount } from "svelte";
  import { ZoomIn, ZoomOut } from "@lucide/svelte";
  import { Button, IconButton, NumberField, SelectField } from "@/shared/ui";
  import type { OutlineItem } from "@/shared/api";
  import type { PDFDocumentLoadingTask, PDFDocumentProxy } from "pdfjs-dist";
  import workerUrl from "pdfjs-dist/build/pdf.worker.min.mjs?url";
  import "pdfjs-dist/web/pdf_viewer.css";
  import { destroyPdf, renderPdfPage as renderPageLayer, selectedPdfText } from "../lib/pdf-renderer";

  let {
    url,
    savedOutline = [],
    initialPage = 1,
    initialZoom = 1,
    initialScroll = 0,
    onPosition,
    onSelection,
    onOutline,
  }: {
    url: string;
    savedOutline?: OutlineItem[];
    initialPage?: number;
    initialZoom?: number;
    initialScroll?: number;
    onPosition: (page: number, zoom: number, scroll: number) => void;
    onSelection: (excerpt: string, context: string) => void;
    onOutline: (outline: OutlineItem[]) => void;
  } = $props();
  let canvas: HTMLCanvasElement;
  let textLayerContainer: HTMLDivElement;
  let scrollContainer: HTMLDivElement;
  let pdfDocument = $state<PDFDocumentProxy | null>(null);
  let loadingTask: PDFDocumentLoadingTask | null = null;
  let page = $state(1);
  let zoom = $state(1);
  let pageCount = $state(0);
  let embeddedOutline = $state<OutlineItem[]>([]);
  let navigationOutline = $derived(savedOutline.length > 0 ? savedOutline : embeddedOutline);
  let error = $state("");
  let rendering = false;
  let pending = false;
  let pdfjs: typeof import("pdfjs-dist");

  onMount(() => {
    void initialize();
    globalThis.document.addEventListener("selectionchange", selectText);
    return () => {
      globalThis.document.removeEventListener("selectionchange", selectText);
      void destroyPdf(loadingTask, textLayerContainer);
    };
  });

  async function initialize() {
    try {
      pdfjs = await import("pdfjs-dist");
      pdfjs.GlobalWorkerOptions.workerSrc = workerUrl;
      page = initialPage;
      zoom = initialZoom;
      loadingTask = pdfjs.getDocument({
        url,
        wasmUrl: "/pdfjs/wasm/",
        isImageDecoderSupported: false,
        isOffscreenCanvasSupported: false,
      });
      pdfDocument = await loadingTask.promise;
      pageCount = pdfDocument.numPages;
      embeddedOutline = await readOutline(pdfDocument);
      if (savedOutline.length === 0 && embeddedOutline.length > 0) onOutline(embeddedOutline);
      await renderPage();
      scrollContainer.scrollTop = initialScroll;
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    }
  }

  async function readOutline(pdf: PDFDocumentProxy) {
    const items = await pdf.getOutline();
    if (!items) return [];
    const resolved: OutlineItem[] = [];
    async function visit(nodes: typeof items, parentId: string | null) {
      for (const item of nodes) {
        let itemId = parentId;
        try {
          const destination = typeof item.dest === "string" ? await pdf.getDestination(item.dest) : item.dest;
          if (destination) {
            const page = (await pdf.getPageIndex(destination[0])) + 1;
            itemId = `pdf-${resolved.length}-${page}`;
            resolved.push({ id: itemId, title: item.title, page, parentId });
          }
        } catch {
          /* A broken destination must not hide valid nested entries. */
        }
        if (item.items.length > 0) await visit(item.items, itemId);
      }
    }
    await visit(items, null);
    return resolved;
  }

  async function renderPage() {
    if (!pdfDocument || rendering) {
      pending = true;
      return;
    }
    rendering = true;
    pending = false;
    try {
      await renderPageLayer(pdfjs, pdfDocument, page, zoom, canvas, textLayerContainer);
    } finally {
      rendering = false;
      if (pending) await renderPage();
    }
  }

  async function go(target: number) {
    page = Math.min(Math.max(target, 1), pageCount || 1);
    scrollContainer.scrollTop = 0;
    await renderPage();
    onPosition(page, zoom, 0);
  }
  async function changeZoom(delta: number) {
    zoom = Math.min(Math.max(zoom + delta, 0.5), 4);
    await renderPage();
    onPosition(page, zoom, scrollContainer.scrollTop);
  }
  function selectText() {
    const selected = selectedPdfText(textLayerContainer);
    if (selected) onSelection(selected.excerpt, selected.context);
  }
</script>

<section class="overflow-hidden border border-[#cfd1cd] bg-[#52565b]" aria-label="Встроенный PDF.js просмотрщик">
  <div
    class="flex items-center gap-[7px] bg-[#27313a] p-2 text-white [&_label]:flex [&_label]:items-center [&_label]:gap-[5px] [&_input]:min-h-10 [&_input]:w-[58px] [&_input]:p-[7px]"
  >
    <Button disabled={page <= 1} onclick={() => go(page - 1)}>Предыдущая</Button>
    <div class="grid grid-cols-[58px_auto] items-end gap-1">
      <NumberField id="pdf-page" label="Страница" min={1} max={pageCount} bind:value={page} onChange={go} />
      <span class="pb-2.5">из {pageCount}</span>
    </div>
    <Button disabled={page >= pageCount} onclick={() => go(page + 1)}>Следующая</Button>
    <IconButton label="Уменьшить масштаб" onclick={() => changeZoom(-0.1)}>
      {#snippet icon()}<ZoomOut size={18} />{/snippet}
    </IconButton>
    <span>{Math.round(zoom * 100)}%</span>
    <IconButton label="Увеличить масштаб" onclick={() => changeZoom(0.1)}>
      {#snippet icon()}<ZoomIn size={18} />{/snippet}
    </IconButton>
    {#if navigationOutline.length}<div class="ml-auto min-w-48 text-ink">
        <SelectField
          label="Оглавление книги"
          placeholder="Оглавление"
          options={navigationOutline.map((item) => ({
            value: String(item.page),
            label: item.parentId ? `↳ ${item.title}` : item.title,
          }))}
          onValueChange={(value) => go(Number(value))}
        />
      </div>{/if}
  </div>
  {#if error}<p role="alert">Не удалось открыть PDF: {error}</p>{/if}
  <div
    class="h-[58vh] min-h-[440px] overflow-auto"
    bind:this={scrollContainer}
    onscroll={() => onPosition(page, zoom, scrollContainer.scrollTop)}
  >
    <div class="relative mx-auto my-[22px] w-max shadow-[0_4px_20px_#0006] [&_canvas]:block">
      <canvas bind:this={canvas}></canvas>
      <div class="textLayer" bind:this={textLayerContainer}></div>
    </div>
  </div>
</section>

<style>
  /* PDF.js owns and positions the dynamic display layer. */
  :global(.textLayer) {
    position: absolute;
    inset: 0;
  }
</style>
