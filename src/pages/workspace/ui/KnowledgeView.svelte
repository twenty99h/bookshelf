<script module lang="ts">
  function assignmentLabel(assignment: string): string {
    return (
      { recall: "Восстановление", transfer: "Передача", experiment: "Практика", mastered: "Освоено" }[assignment] ??
      assignment
    );
  }
</script>

<script lang="ts">
  import { onMount } from "svelte";
  import { resolve } from "$app/paths";
  import { Sparkles, X } from "@lucide/svelte";
  import { Button, CheckboxField, DialogModal, SelectField, TextArea, TextField } from "@/shared/ui";
  import type { Book, Idea, IdeaAssignment, IdeaRelation, LibraryState, ReviewDecision } from "@/shared/api";
  let {
    library,
    selectedIdea,
    bookFilterId,
    selectedTopic = $bindable(),
    bookForIdea,
    onSave,
    onLink,
    onPrepareReview,
    onRunReview,
    onResolveReview,
  }: {
    library: LibraryState;
    selectedIdea: Idea | null;
    bookFilterId: string;
    selectedTopic: string;
    bookForIdea: (idea: Idea) => Book | undefined;
    onSave: (formulation: string, assignments: IdeaAssignment[]) => Promise<void>;
    onLink: (relatedIdeaId: string, relation: IdeaRelation) => Promise<void>;
    onPrepareReview: () => Promise<string>;
    onRunReview: (approvedPackage: string) => Promise<string>;
    onResolveReview: (
      decision: Exclude<ReviewDecision, "pending">,
      formulation: string,
      conclusion: string,
    ) => Promise<boolean>;
  } = $props();

  let formulation = $state("");
  let assignments = $state<IdeaAssignment[]>([]);
  let relatedIdeaId = $state("");
  let relation = $state<IdeaRelation>("complements");
  let detailOpen = $state(true);
  let codexOpen = $state(false);
  let codexPackage = $state("");
  let codexResult = $state("");
  let reviewFormulation = $state("");
  let reviewConclusion = $state("");
  const relatedIdeas = $derived(
    library.ideas.filter((idea) => idea.bookId === selectedIdea?.bookId && idea.id !== selectedIdea?.id),
  );
  const pendingReview = $derived(
    library.reviews.find(
      (review) => review.ideaId === selectedIdea?.id && review.requestKind === "ideaReview" && review.pending,
    ) ?? null,
  );

  onMount(() => {
    formulation = selectedIdea?.formulation ?? "";
    assignments = [...(selectedIdea?.assignments ?? [])];
    reviewFormulation = selectedIdea?.formulation ?? "";
  });

  function focusCompactDrawer(node: HTMLElement) {
    if (matchMedia("(max-width: 1280px)").matches) queueMicrotask(() => node.focus());
  }

  function toggleAssignment(assignment: IdeaAssignment, checked: boolean) {
    assignments = checked
      ? [...new Set([...assignments, assignment])]
      : assignments.filter((candidate) => candidate !== assignment);
  }

  async function prepareReview() {
    codexPackage = await onPrepareReview();
    codexResult = "";
    codexOpen = Boolean(codexPackage);
  }

  async function resolveReview(decision: Exclude<ReviewDecision, "pending">) {
    const saved = await onResolveReview(decision, reviewFormulation, reviewConclusion);
    if (saved && decision !== "later") codexOpen = false;
  }
</script>

<svelte:window
  onkeydown={(event) => {
    if (event.key === "Escape" && matchMedia("(max-width: 1280px)").matches) detailOpen = false;
  }}
/>

<div class="mb-5 flex items-end justify-between">
  <p class="max-w-xl text-sm leading-6 text-mist-dim">
    Авторские формулировки с источниками, назначениями и подтверждёнными связями.
  </p>
  <SelectField
    label="Тема знаний"
    value={selectedTopic}
    options={[
      { value: "all", label: "Все темы" },
      ...library!.topics.map((topic) => ({ value: topic.id, label: topic.name })),
    ]}
    onValueChange={(value) => (selectedTopic = value)}
  />
</div>
<section
  class="grid min-h-[680px] grid-cols-[38%_62%] overflow-hidden rounded-xl border border-white/8 bg-slate max-[1280px]:grid-cols-1"
>
  <div class="border-r border-white/8">
    <div class="border-b border-white/8 p-4">
      <TextField id="knowledge-search" ariaLabel="Поиск по знаниям" placeholder="Найти формулировку" />
    </div>
    {#each library!.ideas.filter((idea) => (!bookFilterId || idea.bookId === bookFilterId) && (selectedTopic === "all" || idea.topicIds.includes(selectedTopic))) as idea (idea.id)}<a
        href={resolve("/knowledge/[ideaId]", { ideaId: idea.id })}
        aria-current={selectedIdea?.id === idea.id ? "true" : undefined}
        class="block border-b border-white/8 p-5 text-mist no-underline hover:bg-white/[.025] aria-[current=true]:border-l-2 aria-[current=true]:border-l-iris aria-[current=true]:bg-iris/[.07]"
        ><small class="font-mono text-[10px] uppercase text-amber">{bookForIdea(idea)?.title} · {idea.section}</small>
        <h2 class="mt-2 line-clamp-3 text-[15px] font-semibold leading-6">{idea.formulation}</h2>
        <div class="mt-3 flex gap-2">
          {#each idea.assignments as assignment (assignment)}<span
              class="rounded bg-night/50 px-2 py-1 font-mono text-[10px] text-mist-dim"
              >{assignmentLabel(assignment)}</span
            >{/each}
        </div></a
      >{/each}
  </div>
  {#if selectedIdea && detailOpen}<article
      {@attach focusCompactDrawer}
      tabindex="-1"
      aria-labelledby="knowledge-detail-title"
      class="overflow-auto p-8 outline-none max-[1281px]:fixed max-[1281px]:inset-y-4 max-[1281px]:right-4 max-[1281px]:z-30 max-[1281px]:w-[min(720px,calc(100vw-2rem))] max-[1281px]:rounded-xl max-[1281px]:border max-[1281px]:border-white/10 max-[1281px]:bg-slate max-[1281px]:shadow-2xl"
    >
      <button
        class="ml-auto hidden size-10 place-items-center rounded-md border border-white/10 max-[1281px]:grid"
        aria-label="Закрыть идею"
        onclick={() => (detailOpen = false)}><X class="size-4" /></button
      >
      <p class="font-mono text-xs uppercase tracking-[.14em] text-iris">Идея книги</p>
      <h2 id="knowledge-detail-title" class="mt-4 max-w-4xl text-3xl font-semibold leading-[1.3] tracking-[-.025em]">
        {selectedIdea.formulation}
      </h2>
      <section class="mt-7 grid gap-4 rounded-lg border border-white/8 bg-night/25 p-5">
        <TextArea id="idea-formulation" label="Самостоятельная формулировка" bind:value={formulation} />
        <div class="grid grid-cols-4 gap-3">
          {#each [["recall", "Восстановление"], ["transfer", "Передача"], ["experiment", "Практика"], ["mastered", "Освоено"]] as assignment (assignment[0])}<CheckboxField
              id={`assignment-${assignment[0]}`}
              label={assignment[1] ?? ""}
              checked={assignments.includes(assignment[0] as IdeaAssignment)}
              onCheckedChange={(checked) => toggleAssignment(assignment[0] as IdeaAssignment, checked)}
            />{/each}
        </div>
        <div>
          <Button
            variant="primary"
            disabled={!formulation.trim() || assignments.length === 0}
            onclick={() => onSave(formulation, assignments)}>Сохранить идею</Button
          >
        </div>
      </section>
      <section class="mt-9">
        <div class="flex items-center justify-between">
          <h3 class="font-semibold">Источники</h3>
          <a class="text-sm text-amber no-underline" href={resolve("/reader/[bookId]", { bookId: selectedIdea.bookId })}
            >Открыть в книге</a
          >
        </div>
        {#each selectedIdea.fragments as fragment (`${fragment.page}-${fragment.excerpt}`)}<blockquote
            class="mt-4 border-l-2 border-amber bg-night/30 p-5"
          >
            <p class="leading-7">{fragment.excerpt}</p>
            <footer class="mt-3 font-mono text-xs text-amber">
              {bookForIdea(selectedIdea)?.title} · стр. {fragment.page}
            </footer>
          </blockquote>{/each}
      </section>
      <section class="mt-8 grid grid-cols-2 gap-5">
        <div class="rounded-lg border border-white/8 p-5">
          <h3 class="font-semibold">Практика</h3>
          <p class="mt-3 text-sm leading-6 text-mist-dim">
            {library!.experiments.find((experiment) => experiment.ideaId === selectedIdea.id)?.situation ??
              "Практический эксперимент ещё не создан."}
          </p>
        </div>
        <div class="rounded-lg border border-white/8 p-5">
          <h3 class="font-semibold">Связи</h3>
          <div class="mt-3 grid gap-3">
            {#each library!.ideaLinks.filter((link) => link.fromIdeaId === selectedIdea.id || link.toIdeaId === selectedIdea.id) as link (link.id)}{@const otherId =
                link.fromIdeaId === selectedIdea.id ? link.toIdeaId : link.fromIdeaId}
              <p class="text-sm text-mist-dim">
                {assignmentLabel(link.relation)} · {library!.ideas.find((idea) => idea.id === otherId)?.formulation}
              </p>{/each}
            <SelectField
              label="Связанная идея"
              value={relatedIdeaId}
              options={relatedIdeas.map((idea) => ({ value: idea.id, label: idea.formulation }))}
              onValueChange={(value) => (relatedIdeaId = value)}
            />
            <SelectField
              label="Подтверждённое отношение"
              value={relation}
              options={[
                { value: "complements", label: "Дополняет" },
                { value: "clarifies", label: "Уточняет" },
                { value: "contradicts", label: "Противоречит" },
              ]}
              onValueChange={(value) => (relation = value as IdeaRelation)}
            />
            <Button disabled={!relatedIdeaId} onclick={() => onLink(relatedIdeaId, relation)}>Подтвердить связь</Button>
          </div>
        </div>
      </section>
      <section class="mt-8 border-t border-white/8 pt-6">
        <h3 class="font-semibold">История версий</h3>
        <ol class="mt-4 grid gap-3">
          {#each selectedIdea.versions.toReversed() as version (`${version.savedAt}-${version.formulation}`)}<li
              class="rounded-md border border-white/8 p-4"
            >
              <time class="font-mono text-xs text-mist-dim"
                >{new Date(version.savedAt * 1000).toLocaleDateString("ru-RU")}</time
              >
              <p class="mt-2 text-sm leading-6">{version.formulation}</p>
            </li>{:else}<li class="text-sm text-mist-dim">Предыдущих версий пока нет.</li>{/each}
        </ol>
      </section>
      <section class="mt-8 border-t border-white/8 pt-6">
        <div class="flex items-center gap-3">
          <Sparkles class="size-5 text-iris" />
          <h3 class="font-semibold">Проверка идеи Codex</h3>
          <span class="ml-auto font-mono text-[10px] uppercase text-mist-faint">Только по явному запросу</span>
        </div>
        <p class="mt-3 max-w-3xl text-sm leading-6 text-mist-dim">
          Перед запуском вы увидите пакет из инструкции, этого источника и своей формулировки. Другие записи и PDF
          целиком не передаются.
        </p>
        <Button onclick={prepareReview}>Подготовить проверку</Button>
      </section>
    </article>{/if}
</section>

<DialogModal
  bind:open={codexOpen}
  title="Проверка идеи Codex"
  description="Проверьте минимальный пакет перед явной отправкой. PDF и другие записи не включены."
>
  {#snippet trigger()}<span class="sr-only">Открыть проверку идеи</span>{/snippet}
  <TextArea id="codex-package" label="Подтверждаемый пакет" bind:value={codexPackage} />
  <Button
    variant="primary"
    disabled={!codexPackage.trim()}
    onclick={async () => (codexResult = await onRunReview(codexPackage))}>Запустить проверку</Button
  >
  {#if pendingReview}<section class="grid gap-4 rounded-md border border-iris/25 bg-night/40 p-4">
      <div>
        <b>Обратная связь</b>
        <p class="mt-2 whitespace-pre-wrap text-sm leading-6">{pendingReview.response}</p>
      </div>
      <TextArea id="review-formulation" label="Авторская формулировка" bind:value={reviewFormulation} />
      <TextArea id="review-conclusion" label="Необязательный авторский вывод" bind:value={reviewConclusion} />
      <div class="flex flex-wrap gap-2">
        <Button variant="primary" disabled={!reviewFormulation.trim()} onclick={() => resolveReview("refined")}
          >Уточнить идею</Button
        ><Button onclick={() => resolveReview("unchanged")}>Оставить без изменений</Button><Button
          onclick={() => resolveReview("later")}>Вернуться позже</Button
        >
      </div>
    </section>{/if}
  {#if codexResult}<p class="rounded-md border border-white/8 bg-night/40 p-4 text-sm" role="status">
      {codexResult}
    </p>{/if}
</DialogModal>
