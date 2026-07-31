<script lang="ts">
  import { resolve } from "$app/paths";
  import { BookOpen, ChevronRight, Plus } from "@lucide/svelte";
  import { Button } from "@/shared/ui";
  import type { Book, LibraryState } from "@/shared/api";

  let {
    library,
    activeBook,
    unfinishedCount,
    busy,
    onImport,
  }: {
    library: LibraryState;
    activeBook: Book | null;
    unfinishedCount: number;
    busy: boolean;
    onImport: () => void;
  } = $props();

  const latestMilestoneAt = $derived(Math.max(0, ...library.milestones.map((milestone) => milestone.occurredAt)));
  const weekStart = $derived(Math.max(0, latestMilestoneAt - 7 * 86_400));
  const weeklyMilestones = $derived(library.milestones.filter((milestone) => milestone.occurredAt >= weekStart));
  const weeklyReadingPages = $derived(
    weeklyMilestones
      .filter((milestone) => milestone.kind === "readingProgress" && milestone.page !== null)
      .map((milestone) => milestone.page!)
      .toSorted((a, b) => a - b),
  );
  const weeklyReadingProgress = $derived(
    weeklyReadingPages.length > 1 ? weeklyReadingPages.at(-1)! - weeklyReadingPages[0]! : 0,
  );
  const weeklyIdeaCount = $derived(weeklyMilestones.filter((milestone) => milestone.kind === "ideaFormulated").length);
  const weeklyRecallCount = $derived(
    weeklyMilestones.filter((milestone) => milestone.kind === "recallCompleted").length,
  );
  const weeklyExperimentCount = $derived(
    weeklyMilestones.filter((milestone) => milestone.kind === "experimentAdvanced").length,
  );
  const weeklyRange = $derived.by(() => {
    if (!latestMilestoneAt) return "Нет вех за неделю";
    const format = new Intl.DateTimeFormat("ru-RU", { day: "numeric", month: "short" });
    return `${format.format(new Date(weekStart * 1_000))}–${format.format(new Date(latestMilestoneAt * 1_000))}`;
  });
</script>

{#if library.books.length === 0}
  <section class="grid min-h-[72vh] place-items-center">
    <div class="max-w-xl text-center">
      <div
        class="mx-auto mb-8 grid h-32 w-24 place-items-center rounded-r-xl border border-amber/30 bg-slate shadow-[inset_7px_0_0_#d6a24a]"
      >
        <BookOpen class="size-10 text-amber" />
      </div>
      <p class="mb-3 font-mono text-xs uppercase tracking-[0.18em] text-amber">
        Личная библиотека · только на этом компьютере
      </p>
      <h2 class="text-4xl font-semibold tracking-[-0.03em]">Начните с одной книги, к которой хотите возвращаться</h2>
      <p class="mx-auto mt-5 max-w-lg text-base leading-7 text-mist-dim">
        Bookshelf сохраняет PDF, место чтения и ваши формулировки локально. Облачной синхронизации и обязательной
        настройки нет.
      </p>
      <div class="mt-8">
        <Button variant="primary" disabled={busy} onclick={onImport}
          ><Plus class="mr-2 size-4" />Импортировать PDF</Button
        >
      </div>
    </div>
  </section>
{:else}
  <section class="grid grid-cols-[minmax(0,1.55fr)_minmax(330px,.75fr)] gap-6 max-[1280px]:grid-cols-1">
    <div class="grid gap-6">
      <article class="relative overflow-hidden rounded-xl border border-white/8 bg-slate p-8">
        <div class="absolute inset-y-0 left-0 w-1 bg-iris"></div>
        <p class="font-mono text-xs uppercase tracking-[0.15em] text-iris">Активное изучение</p>
        <h2 class="mt-4 max-w-3xl text-4xl font-semibold tracking-[-0.035em]">{activeBook?.title}</h2>
        <p class="mt-3 text-mist-dim">
          Глава 5 · Репликация · последняя позиция <span class="font-mono text-mist">{activeBook?.reading.page}</span>
        </p>
        <div class="mt-9 flex items-center gap-3">
          <a
            class="inline-flex min-h-11 items-center gap-2 rounded-md bg-iris-strong px-5 font-semibold text-white no-underline hover:bg-[#4d48aa] focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-iris"
            href={resolve("/reader/[bookId]", { bookId: activeBook?.id ?? "book" })}
            ><BookOpen class="size-4" />Продолжить чтение</a
          >
          <a
            class="inline-flex min-h-11 items-center rounded-md border border-white/12 px-4 text-sm text-mist no-underline hover:bg-white/5"
            href={resolve("/library/[bookId]", { bookId: activeBook?.id ?? "book" })}>Открыть книгу</a
          >
        </div>
        <div class="mt-8 grid grid-cols-3 gap-6 border-t border-white/8 pt-6">
          {@render metric("Последняя позиция", `${activeBook?.reading.page}`, "страница")}
          {@render metric("Пройдено по тексту", `${activeBook?.farthestPage ?? 0}`, "дальняя позиция")}
          {@render metric(
            "Черновики книги",
            `${library.drafts.filter((draft) => draft.bookId === activeBook?.id).length}`,
            "на разбор",
          )}
        </div>
      </article>
      <article class="rounded-xl border border-white/8 bg-slate p-6">
        <div class="flex items-end justify-between gap-4">
          <div>
            <p class="font-mono text-xs uppercase tracking-[0.15em] text-mist-dim">Последние семь дней</p>
            <h2 class="mt-2 text-xl font-semibold">Текст переходит в знание</h2>
          </div>
          <span class="font-mono text-xs text-mist-dim">{weeklyRange}</span>
        </div>
        <div class="mt-6 grid grid-cols-4 gap-px overflow-hidden rounded-lg border border-white/8 bg-white/8">
          {@render weeklyCell("Текст", `+${weeklyReadingProgress} стр.`, "Самая дальняя позиция")}{@render weeklyCell(
            "Идеи",
            `${weeklyIdeaCount}`,
            "Сформулировано",
          )}{@render weeklyCell("Восстановления", `${weeklyRecallCount}`, "Решения читателя")}{@render weeklyCell(
            "Практика",
            `${weeklyExperimentCount}`,
            "Продвижение",
          )}
        </div>
      </article>
    </div>
    <aside class="grid content-start gap-6">
      <article class="rounded-xl border border-white/8 bg-slate p-6">
        <div class="flex items-center justify-between">
          <div>
            <p class="font-mono text-xs uppercase tracking-[0.15em] text-mist-dim">Незавершённая работа</p>
            <h2 class="mt-2 text-xl font-semibold">{unfinishedCount} требуют решения</h2>
          </div>
          <span class="grid size-10 place-items-center rounded-full bg-amber/12 font-mono text-amber"
            >{unfinishedCount}</span
          >
        </div>
        <div class="mt-5 grid gap-1">
          {@render workRow("Черновые заметки", `${library.drafts.length}`, "/drafts")}
          {@render workRow(
            "Проверки идей",
            `${library.reviews.filter((review) => review.pending).length}`,
            "/knowledge",
          )}
          {@render workRow(
            "Эксперименты",
            `${library.experiments.filter((experiment) => !["completed", "cancelled"].includes(experiment.status)).length}`,
            "/practice",
          )}
        </div>
        <p class="mt-5 text-sm leading-6 text-mist-dim">Эта работа сохранена и не мешает продолжить книгу.</p>
      </article>
      <article class="rounded-xl border border-white/8 bg-night/40 p-6">
        <p class="font-mono text-xs uppercase tracking-[0.15em] text-mist-dim">Рекомендуемый следующий шаг</p>
        <h3 class="mt-3 font-semibold">Разобрать источник о смене лидера</h3>
        <p class="mt-2 text-sm leading-6 text-mist-dim">
          Свежий фрагмент связан с текущей главой и займёт одно решение.
        </p>
        <a
          href={resolve("/drafts")}
          class="mt-4 inline-flex items-center gap-1 text-sm font-semibold text-iris no-underline"
          >Открыть черновик <ChevronRight class="size-4" /></a
        >
      </article>
    </aside>
  </section>
{/if}

{#snippet metric(label: string, value: string, detail: string)}<div>
    <span class="block font-mono text-2xl text-mist">{value}</span><span class="mt-1 block text-sm text-mist-dim"
      >{label}</span
    ><small class="font-mono text-[10px] uppercase text-mist-faint">{detail}</small>
  </div>{/snippet}
{#snippet weeklyCell(label: string, value: string, detail: string)}<div class="bg-night/30 p-4">
    <span class="font-mono text-xl">{value}</span><b class="mt-2 block text-sm">{label}</b><small class="text-mist-dim"
      >{detail}</small
    >
  </div>{/snippet}
{#snippet workRow(label: string, value: string, href: "/drafts" | "/knowledge" | "/practice")}<a
    href={resolve(href)}
    class="flex items-center rounded-md px-2 py-3 text-sm text-mist no-underline hover:bg-white/5"
    ><span>{label}</span><span class="ml-auto font-mono text-mist-dim">{value}</span><ChevronRight
      class="ml-2 size-4 text-mist-faint"
    /></a
  >{/snippet}
