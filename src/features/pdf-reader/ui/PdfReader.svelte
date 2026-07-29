<script lang="ts">
  import { onMount } from "svelte";
  import { Button } from "@/shared/ui";
  import type { PDFDocumentProxy } from "pdfjs-dist";
  import workerUrl from "pdfjs-dist/build/pdf.worker.min.mjs?url";
  import "pdfjs-dist/web/pdf_viewer.css";

  let { url, savedOutline = [], initialPage = 1, initialZoom = 1, initialScroll = 0, onPosition, onSelection, onOutline }: {
    url: string;
    savedOutline?: { id: string; title: string; page: number; parentId?: string | null }[];
    initialPage?: number;
    initialZoom?: number;
    initialScroll?: number;
    onPosition: (page: number, zoom: number, scroll: number) => void;
    onSelection: (excerpt: string, context: string) => void;
    onOutline: (outline: { id: string; title: string; page: number; parentId: null }[]) => void;
  } = $props();
  let canvas: HTMLCanvasElement;
  let textLayerContainer: HTMLDivElement;
  let scrollContainer: HTMLDivElement;
  let pdfDocument = $state<PDFDocumentProxy | null>(null);
  let page = $state(1);
  let zoom = $state(1);
  let pageCount = $state(0);
  let embeddedOutline = $state<{ id: string; title: string; page: number; parentId: null }[]>([]);
  let navigationOutline = $derived(savedOutline.length > 0 ? savedOutline : embeddedOutline);
  let error = $state("");
  let rendering = false;
  let pending = false;
  let pdfjs: typeof import("pdfjs-dist");

  onMount(() => {
    void initialize();
    globalThis.document.addEventListener("selectionchange", selectText);
    return () => globalThis.document.removeEventListener("selectionchange", selectText);
  });

  async function initialize() {
    try {
      pdfjs = await import("pdfjs-dist");
      pdfjs.GlobalWorkerOptions.workerSrc = workerUrl;
      page = initialPage;
      zoom = initialZoom;
      pdfDocument = await pdfjs.getDocument({ url }).promise;
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
    const resolved = await Promise.all(items.map(async (item, index) => {
      try {
        const destination = typeof item.dest === "string" ? await pdf.getDestination(item.dest) : item.dest;
        if (!destination) return null;
        return { id: `pdf-${index}-${(await pdf.getPageIndex(destination[0])) + 1}`, title: item.title, page: (await pdf.getPageIndex(destination[0])) + 1, parentId: null };
      } catch { return null; }
    }));
    return resolved.filter((item): item is { id: string; title: string; page: number; parentId: null } => item !== null);
  }

  async function renderPage() {
    if (!pdfDocument || rendering) { pending = true; return; }
    rendering = true; pending = false;
    try {
      const pdfPage = await pdfDocument.getPage(page);
      const viewport = pdfPage.getViewport({ scale: zoom });
      const ratio = window.devicePixelRatio || 1;
      canvas.width = Math.floor(viewport.width * ratio); canvas.height = Math.floor(viewport.height * ratio);
      canvas.style.width = `${viewport.width}px`; canvas.style.height = `${viewport.height}px`;
      textLayerContainer.replaceChildren();
      textLayerContainer.style.width = `${viewport.width}px`; textLayerContainer.style.height = `${viewport.height}px`;
      await pdfPage.render({ canvas, canvasContext: canvas.getContext("2d")!, viewport, transform: ratio === 1 ? undefined : [ratio, 0, 0, ratio, 0, 0] }).promise;
      const text = await pdfPage.getTextContent();
      await new pdfjs.TextLayer({ textContentSource: text, container: textLayerContainer, viewport }).render();
    } finally {
      rendering = false;
      if (pending) await renderPage();
    }
  }

  async function go(target: number) { page = Math.min(Math.max(target, 1), pageCount || 1); scrollContainer.scrollTop = 0; await renderPage(); onPosition(page, zoom, 0); }
  async function changeZoom(delta: number) { zoom = Math.min(Math.max(zoom + delta, .5), 4); await renderPage(); onPosition(page, zoom, scrollContainer.scrollTop); }
  function selectText() {
    const selection = window.getSelection();
    const excerpt = selection?.toString().trim() ?? "";
    if (!excerpt || !selection?.anchorNode || !textLayerContainer.contains(selection.anchorNode)) return;
    const context = textLayerContainer.textContent?.replace(/\s+/g, " ").trim() ?? "";
    onSelection(excerpt, context.slice(Math.max(0, context.indexOf(excerpt) - 180), context.indexOf(excerpt) + excerpt.length + 180));
  }
</script>

<section class="pdf-viewer" aria-label="Встроенный PDF.js просмотрщик">
  <div class="pdf-toolbar"><Button disabled={page <= 1} onclick={() => go(page - 1)}>Предыдущая</Button><label>Страница <input aria-label="Текущая страница" type="number" min="1" max={pageCount} bind:value={page} onchange={() => go(page)} /> из {pageCount}</label><Button disabled={page >= pageCount} onclick={() => go(page + 1)}>Следующая</Button><Button aria-label="Уменьшить масштаб" onclick={() => changeZoom(-.1)}>−</Button><span>{Math.round(zoom * 100)}%</span><Button aria-label="Увеличить масштаб" onclick={() => changeZoom(.1)}>+</Button>{#if navigationOutline.length}<select aria-label="Оглавление книги" onchange={(event) => go(Number(event.currentTarget.value))}><option value="">Оглавление</option>{#each navigationOutline as item}<option value={item.page}>{item.parentId ? `↳ ${item.title}` : item.title}</option>{/each}</select>{/if}</div>
  {#if error}<p role="alert">Не удалось открыть PDF: {error}</p>{/if}
  <div class="pdf-scroll" bind:this={scrollContainer} onscroll={() => onPosition(page, zoom, scrollContainer.scrollTop)}><div class="pdf-page"><canvas bind:this={canvas}></canvas><div class="textLayer" bind:this={textLayerContainer}></div></div></div>
</section>

<style>
  .pdf-viewer { overflow: hidden; border: 1px solid #cfd1cd; background: #52565b; }
  .pdf-toolbar { display: flex; align-items: center; gap: 7px; background: #27313a; padding: 8px; color: white; }
  .pdf-toolbar label { display: flex; align-items: center; gap: 5px; }
  .pdf-toolbar input { width: 58px; min-height: 40px; padding: 7px; }
  .pdf-toolbar select { width: auto; margin: 0; }
  .pdf-scroll { height: 58vh; min-height: 440px; overflow: auto; }
  .pdf-page { position: relative; width: max-content; margin: 22px auto; box-shadow: 0 4px 20px #0006; }
  .pdf-page canvas { display: block; }
  .pdf-page :global(.textLayer) { position: absolute; inset: 0; }
</style>
