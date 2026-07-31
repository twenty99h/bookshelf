<script lang="ts">
  import { onMount } from "svelte";
  import type { PDFDocumentLoadingTask, PDFDocumentProxy } from "pdfjs-dist";
  import workerUrl from "pdfjs-dist/build/pdf.worker.min.mjs?url";
  import type { OutlineItem, SourceFragment } from "@/shared/api";
  import PdfPage from "./PdfPage.svelte";

  let {
    url,
    initialPage,
    initialScroll,
    zoom,
    mode,
    invertImages,
    onPosition,
    onSelection,
    onOutline,
    searchQuery,
    onSearchResults,
    sources,
    onSourceSelect,
  }: {
    url: string;
    initialPage: number;
    initialScroll: number;
    zoom: number;
    mode: "muted" | "original" | "dark";
    invertImages: boolean;
    onPosition: (page: number, scroll: number) => void;
    onSelection: (fragments: SourceFragment[]) => void;
    onOutline: (outline: OutlineItem[]) => void;
    searchQuery: string;
    onSearchResults: (results: { page: number; excerpt: string }[]) => void;
    sources: SourceFragment[];
    onSourceSelect: (source: SourceFragment) => void;
  } = $props();

  let loadingTask: PDFDocumentLoadingTask | null = null;
  let pdfDocument = $state<PDFDocumentProxy | null>(null);
  let pageCount = $state(0);
  let currentPage = $state(1);
  let error = $state("");
  let container: HTMLDivElement | null = null;
  let saveTimer: ReturnType<typeof setTimeout> | null = null;
  let searchGeneration = 0;
  const estimatedPageHeight = $derived(842 * zoom + 20);
  const firstPage = $derived(Math.max(1, currentPage - 2));
  const lastPage = $derived(Math.min(pageCount, currentPage + 2));
  const visiblePages = $derived(
    Array.from({ length: Math.max(0, lastPage - firstPage + 1) }, (_, index) => firstPage + index),
  );

  function rememberContainer(node: HTMLDivElement) {
    container = node;
    return () => {
      if (container === node) container = null;
    };
  }

  onMount(() => {
    currentPage = initialPage;
    void initialize();
    globalThis.document.addEventListener("selectionchange", captureSelection);
    return () => {
      globalThis.document.removeEventListener("selectionchange", captureSelection);
      if (saveTimer) clearTimeout(saveTimer);
      void loadingTask?.destroy();
    };
  });

  function searchAttachment() {
    const query = searchQuery.trim();
    const pdf = pdfDocument;
    const generation = ++searchGeneration;
    if (!query || !pdf) {
      queueMicrotask(() => onSearchResults([]));
    } else {
      void searchDocument(pdf, query, generation);
    }
    return () => {
      if (generation === searchGeneration) searchGeneration += 1;
    };
  }

  async function initialize() {
    try {
      const pdfjs = await import("pdfjs-dist");
      pdfjs.GlobalWorkerOptions.workerSrc = workerUrl;
      loadingTask = pdfjs.getDocument({
        url,
        wasmUrl: "/pdfjs/wasm/",
        isImageDecoderSupported: false,
        isOffscreenCanvasSupported: false,
      });
      pdfDocument = await loadingTask.promise;
      pageCount = pdfDocument.numPages;
      currentPage = Math.min(pageCount, Math.max(1, initialPage));
      onOutline(await readOutline(pdfDocument));
      onPosition(currentPage, initialScroll);
      requestAnimationFrame(() => {
        if (container) container.scrollTop = (currentPage - 1 + initialScroll) * estimatedPageHeight;
      });
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    }
  }

  async function readOutline(pdf: PDFDocumentProxy): Promise<OutlineItem[]> {
    const outline = await pdf.getOutline();
    if (!outline) return [];
    const result: OutlineItem[] = [];
    for (const [index, item] of outline.entries()) {
      try {
        const destination = typeof item.dest === "string" ? await pdf.getDestination(item.dest) : item.dest;
        if (!destination) continue;
        const page = (await pdf.getPageIndex(destination[0])) + 1;
        result.push({ id: `pdf-${index}-${page}`, title: item.title, page, parentId: null });
      } catch {
        // A broken outline entry must not prevent reading the document.
      }
    }
    return result;
  }

  async function searchDocument(pdf: PDFDocumentProxy, query: string, generation: number) {
    const needle = query.toLocaleLowerCase("ru");
    const results: { page: number; excerpt: string }[] = [];
    for (let pageNumber = 1; pageNumber <= pdf.numPages; pageNumber += 1) {
      const page = await pdf.getPage(pageNumber);
      const content = await page.getTextContent();
      const text = content.items
        .map((item) => ("str" in item ? item.str : ""))
        .join(" ")
        .replace(/\s+/g, " ");
      const index = text.toLocaleLowerCase("ru").indexOf(needle);
      if (index >= 0)
        results.push({ page: pageNumber, excerpt: text.slice(Math.max(0, index - 70), index + query.length + 120) });
      if (generation !== searchGeneration) return;
      if (results.length >= 50) break;
    }
    if (generation === searchGeneration) onSearchResults(results);
  }

  function handleScroll() {
    if (!container || !pageCount) return;
    currentPage = Math.min(pageCount, Math.max(1, Math.floor(container.scrollTop / estimatedPageHeight) + 1));
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      if (!container) return;
      const offset = (container.scrollTop % estimatedPageHeight) / estimatedPageHeight;
      onPosition(currentPage, offset);
    }, 250);
  }

  function captureSelection() {
    const selection = window.getSelection();
    if (
      !selection ||
      selection.isCollapsed ||
      !container ||
      !selection.anchorNode ||
      !container.contains(selection.anchorNode)
    )
      return;
    const excerpt = selection.toString().trim();
    if (!excerpt) return;
    const range = selection.getRangeAt(0);
    const fragments = [...container.querySelectorAll<HTMLElement>("[data-pdf-page]")]
      .filter((page) => range.intersectsNode(page))
      .map((page) => ({
        page: Number(page.dataset.pdfPage),
        excerpt,
        context: page.textContent?.replace(/\s+/g, " ").trim().slice(0, 500) ?? "",
      }));
    if (fragments.length) onSelection(fragments);
  }
</script>

<div
  class="h-full overflow-y-auto bg-[#15191f] px-12 py-8 max-[1280px]:px-6"
  {@attach rememberContainer}
  {@attach searchAttachment}
  onscroll={handleScroll}
  data-testid="continuous-pdf"
>
  {#if error}<div class="mx-auto mt-16 max-w-xl rounded-lg border border-danger/40 bg-slate p-6" role="alert">
      <h2 class="font-semibold">Не удалось открыть PDF</h2>
      <p class="mt-2 text-sm text-mist-dim">{error}</p>
    </div>{:else if pdfDocument}<div style:height={`${(firstPage - 1) * estimatedPageHeight}px`}></div>
    <div class="grid gap-5">
      {#each visiblePages as page (page)}<PdfPage
          document={pdfDocument}
          {page}
          {zoom}
          {mode}
          {invertImages}
          sources={sources.filter((source) => source.page === page)}
          {onSourceSelect}
        />{/each}
    </div>
    <div style:height={`${Math.max(0, pageCount - lastPage) * estimatedPageHeight}px`}></div>{:else}<div
      class="grid min-h-[60vh] place-items-center"
      role="status"
    >
      Подготавливаем страницы…
    </div>{/if}
</div>
