<script lang="ts">
  import { onMount, type Snippet } from "svelte";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { page } from "$app/state";
  import { BookCopy, Brain, Command, FlaskConical, Gauge, Library, Settings, StickyNote } from "@lucide/svelte";
  import { provideWorkspaceSession, type WorkspaceContext } from "@/pages/workspace";
  import type { LibraryState } from "@/shared/api";
  import CommandPalette from "./CommandPalette.svelte";

  let { children }: { children: Snippet } = $props();
  const session = provideWorkspaceSession();
  let paletteOpen = $state(false);
  let paletteQuery = $state("");
  let paletteResults = $state<{ id: string; kind: string; title: string; context: string }[]>([]);
  const context = $derived(contextForPath(page.url.pathname));
  const resourceId = $derived(page.url.pathname.split("/").filter(Boolean).at(-1));
  const title = $derived(contextTitle(context, resourceId, session.library));

  onMount(async () => {
    await session.load();
  });

  function contextForPath(pathname: string): WorkspaceContext {
    if (pathname === "/") return "dashboard";
    if (pathname === "/library") return "library";
    if (pathname.endsWith("/complete")) return "completion";
    if (pathname.startsWith("/library/")) return "book";
    if (pathname.startsWith("/drafts")) return "drafts";
    if (pathname.startsWith("/knowledge/")) return "idea";
    if (pathname.startsWith("/knowledge")) return "knowledge";
    if (pathname.startsWith("/practice")) return "practice";
    if (pathname.startsWith("/reader/")) return "reader";
    return "settings";
  }

  function contextTitle(current: WorkspaceContext, id: string | undefined, state: LibraryState | null) {
    if (current === "book") return state?.books.find((book) => book.id === id)?.title ?? "Книга";
    if (current === "idea") return "Идея книги";
    return {
      dashboard: "Рабочий стол",
      library: "Личная библиотека",
      drafts: "Разбор черновиков",
      knowledge: "Знания",
      practice: "Практика",
      completion: "Завершение изучения",
      settings: "Настройки",
      reader: "Режим чтения",
      book: "Книга",
      idea: "Идея книги",
    }[current];
  }

  async function searchPalette() {
    if (!session.commands) return;
    paletteResults = await session.commands.search(paletteQuery);
  }

  function openPaletteResult(result: { id: string; kind: string }) {
    paletteOpen = false;
    if (result.kind === "book") void goto(resolve("/library/[bookId]", { bookId: result.id }));
    else if (result.kind === "idea") void goto(resolve("/knowledge/[ideaId]", { ideaId: result.id }));
    else if (result.kind === "topic") void goto(resolve(`/knowledge?topic=${encodeURIComponent(result.id)}`));
    else if (result.kind === "draft") void goto(resolve(`/drafts?draft=${encodeURIComponent(result.id)}`));
    else if (result.kind === "material") {
      const material = session.library?.materials.find((item) => item.id === result.id);
      const idea = session.library?.ideas.find((item) => material?.ideaIds.includes(item.id));
      const source = idea?.fragments[0];
      if (idea && source) void goto(resolve(`/reader/${encodeURIComponent(idea.bookId)}?sourcePage=${source.page}`));
    }
  }
</script>

<svelte:window
  onkeydown={(event) => {
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
      event.preventDefault();
      paletteOpen = true;
    }
  }}
/>

{#if context === "reader"}
  {@render children()}
{:else}<div class="min-h-screen bg-night text-mist">
    <div class="grid min-h-screen grid-cols-[248px_minmax(0,1fr)] max-[1100px]:grid-cols-[210px_minmax(0,1fr)]">
      <aside
        class="sticky top-0 flex h-screen flex-col border-r border-white/8 bg-night px-4 py-5"
        aria-label="Основная навигация"
      >
        <a href={resolve("/")} class="mb-8 flex items-center gap-3 rounded-lg px-3 py-2 text-mist no-underline"
          ><span class="grid size-9 place-items-center rounded-md border border-amber/35 bg-amber/10 text-amber"
            ><BookCopy class="size-5" /></span
          ><span><b class="block tracking-wide">Bookshelf</b><small class="text-mist-dim">Личное изучение</small></span
          ></a
        >
        <nav class="grid gap-1">
          {@render navItem("dashboard", "/", "Рабочий стол", Gauge)}
          {@render navItem("library", "/library", "Библиотека", Library)}
          {@render navItem("drafts", "/drafts", "Черновики", StickyNote, session.library?.drafts.length)}
          {@render navItem("knowledge", "/knowledge", "Знания", Brain)}
          {@render navItem("practice", "/practice", "Практика", FlaskConical)}
        </nav>
        <div class="mt-auto grid gap-2">
          <button
            class="flex min-h-10 items-center gap-3 rounded-md px-3 text-left text-sm text-mist-dim hover:bg-slate hover:text-mist"
            onclick={() => (paletteOpen = true)}
            ><Command class="size-4" /><span>Быстрый переход</span><kbd class="ml-auto font-mono text-[11px]"
              >Ctrl K</kbd
            ></button
          >
          {@render navItem("settings", "/settings", "Настройки", Settings)}
        </div>
      </aside>
      <main class="min-w-0 bg-graphite">
        <header class="flex min-h-20 items-center justify-between border-b border-white/8 px-8 max-[1280px]:px-6">
          <div>
            <p class="mb-1 font-mono text-[11px] uppercase tracking-[0.16em] text-mist-dim">Bookshelf / {context}</p>
            <h1 class="text-xl font-semibold tracking-tight">{title}</h1>
          </div>
          <span class="rounded-md border border-white/10 bg-slate px-3 py-2 font-mono text-xs text-mist-dim"
            >Локальная библиотека</span
          >
        </header>
        {@render children()}
      </main>
    </div>
  </div>{/if}

<CommandPalette
  bind:open={paletteOpen}
  bind:query={paletteQuery}
  results={paletteResults}
  onSearch={searchPalette}
  onOpenResult={openPaletteResult}
/>

{#snippet navItem(
  itemContext: WorkspaceContext,
  href: "/" | "/library" | "/drafts" | "/knowledge" | "/practice" | "/settings",
  label: string,
  Icon: typeof Gauge,
  badge?: number,
)}
  <a
    href={resolve(href)}
    aria-current={context === itemContext || (itemContext === "knowledge" && context === "idea") ? "page" : undefined}
    class="group flex min-h-11 items-center gap-3 rounded-md border border-transparent px-3 text-sm text-mist-dim no-underline hover:bg-slate hover:text-mist aria-[current=page]:border-iris/20 aria-[current=page]:bg-iris/12 aria-[current=page]:text-mist"
    ><Icon class="size-[18px] group-aria-[current=page]:text-iris" /><span>{label}</span>{#if badge}<span
        class="ml-auto rounded-full bg-amber/15 px-2 py-0.5 font-mono text-[11px] text-amber">{badge}</span
      >{/if}</a
  >
{/snippet}
