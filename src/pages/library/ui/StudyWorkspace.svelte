<script lang="ts">
  import { Button, CheckboxField, SelectField, TextArea, TextField } from "@/shared/ui";
  import type { Book, LibraryAction, LibraryState, SessionStatus } from "@/shared/api";

  let {
    library,
    activeBook,
    run,
    onFeedback,
  }: {
    library: LibraryState;
    activeBook?: Book;
    run: (action: LibraryAction, success?: string) => Promise<void>;
    onFeedback: (message: string) => void;
  } = $props();

  let sessionIntention = $state("");
  let rescheduledSessionStatus = $state<SessionStatus>("moved");
  let rescheduledSessionReason = $state("");
  let retrospective = $state("");
  let significantIdeaIds = $state<string[]>([]);
  let continuingWork = $state("");
  let debtDecision = $state("");

  function sessionStatusLabel(status: SessionStatus) {
    return (
      {
        planned: "Запланирован",
        active: "Сеанс идёт",
        completed: "Завершён",
        moved: "Перенесён",
        replaced: "Заменён",
        cancelled: "Отменён",
      } satisfies Record<SessionStatus, string>
    )[status];
  }

  async function completeSession(sessionId: string) {
    await run({ kind: "resolveSession", sessionId, status: "completed", reason: "" });
    const change = library.lastDebtChange;
    onFeedback(
      `Сеанс завершён. Долг ${change > 0 ? `вырос на ${change}` : change < 0 ? `уменьшился на ${Math.abs(change)}` : "не изменился"}`,
    );
  }

  function setSignificantIdea(ideaId: string, checked: boolean) {
    significantIdeaIds = checked ? [...significantIdeaIds, ideaId] : significantIdeaIds.filter((id) => id !== ideaId);
  }
</script>

<section class="mb-5 grid grid-cols-2 gap-4 max-[640px]:grid-cols-1">
  <article class="rounded-[11px] border border-rule bg-paper-raised p-6 shadow-paper">
    <p class="mb-[7px] text-[11px] font-extrabold uppercase tracking-[.11em] text-[#66717a]">Недельный ритм</p>
    <h2>{library.weeklySessionBudget || 3} сеанса</h2>
    <p>Без дедлайна книги, нормы страниц и штрафов.</p>
    <div class="flex gap-[7px]">
      {#each [2, 3, 4, 5] as count (count)}
        <button
          class={library.weeklySessionBudget === count
            ? "size-[42px] rounded-full border border-[#72843d] bg-[#eaf1c8] font-extrabold"
            : "size-[42px] rounded-full border border-rule bg-white"}
          onclick={() => run({ kind: "setStudyRhythm", weeklySessionBudget: count }, "Недельный ритм сохранён")}
          >{count}</button
        >
      {/each}
    </div>
  </article>
  <article class="rounded-[11px] border border-rule bg-paper-raised p-6 shadow-paper">
    <p class="mb-[7px] text-[11px] font-extrabold uppercase tracking-[.11em] text-[#66717a]">Новый сеанс</p>
    <h2>С каким намерением?</h2>
    <form
      onsubmit={(event) => {
        event.preventDefault();
        run(
          { kind: "planSession", intention: sessionIntention, plannedAt: Math.floor(Date.now() / 1000) },
          "Сеанс запланирован",
        );
        sessionIntention = "";
      }}
    >
      <TextArea
        id="session-intention"
        ariaLabel="Намерение сеанса"
        bind:value={sessionIntention}
        placeholder="Разобрать две заметки и продолжить главу"
        required
      />
      <Button type="submit">Запланировать</Button>
    </form>
  </article>
</section>
{#if library.sessions.length}
  <section class="grid gap-3.5">
    {#each library.sessions as session (session.id)}
      <article
        class="flex items-center justify-between gap-5 border-b border-rule px-[5px] py-[15px] max-[640px]:flex-col max-[640px]:items-stretch [&_small]:mt-1 [&_small]:block [&_small]:text-ink-muted"
      >
        <div><b>{session.intention}</b><small>{sessionStatusLabel(session.status)}</small></div>
        {#if session.status === "planned"}
          <div
            class="flex items-center gap-[7px] max-[640px]:flex-col max-[640px]:items-stretch [&_input]:w-[210px] max-[640px]:[&_input]:w-full"
          >
            <Button
              variant="primary"
              onclick={() =>
                run(
                  { kind: "startSession", sessionId: session.id },
                  "Сеанс начат; изменение долга будет измерено отсюда",
                )}>Начать сеанс</Button
            >
            <SelectField
              label="Решение по пропущенному сеансу"
              bind:value={rescheduledSessionStatus}
              options={[
                { value: "moved", label: "Перенести" },
                { value: "replaced", label: "Заменить" },
                { value: "cancelled", label: "Отменить" },
              ]}
            />
            <TextField
              id={`session-reason-${session.id}`}
              ariaLabel="Причина решения"
              bind:value={rescheduledSessionReason}
              placeholder="Почему план изменился"
            />
            <Button
              onclick={() =>
                run(
                  {
                    kind: "resolveSession",
                    sessionId: session.id,
                    status: rescheduledSessionStatus,
                    reason: rescheduledSessionReason,
                  },
                  "Решение по сеансу сохранено без штрафа",
                )}>Сохранить решение</Button
            >
          </div>
        {:else if session.status === "active"}
          <Button onclick={() => completeSession(session.id)}>Завершить сеанс</Button>
        {/if}
      </article>
    {/each}
  </section>
{/if}
{#if activeBook}
  <form
    class="mt-5 grid gap-[7px] rounded-[11px] border border-rule bg-paper-raised p-6 shadow-paper"
    onsubmit={(event) => {
      event.preventDefault();
      run(
        {
          kind: "completeStudy",
          bookId: activeBook!.id,
          retrospective,
          significantIdeaIds,
          continuingWork,
          debtDecision,
        },
        "Изучение завершено; продолжающаяся работа осталась доступна",
      );
    }}
  >
    <p class="mb-[7px] text-[11px] font-extrabold uppercase tracking-[.11em] text-[#66717a]">
      Явное завершение изучения
    </p>
    <h2>Ретроспектива «{activeBook.title}»</h2>
    <TextArea
      id="retrospective"
      label="Результаты применения и изменения в понимании или действиях"
      bind:value={retrospective}
    />
    <fieldset>
      <legend>3–7 значимых идей</legend>
      <div class="grid gap-2 py-2">
        {#each library.ideas.filter((idea) => idea.bookId === activeBook?.id) as idea (idea.id)}
          <CheckboxField
            label={idea.formulation}
            checked={significantIdeaIds.includes(idea.id)}
            onCheckedChange={(checked) => setSignificantIdea(idea.id, checked)}
          />
        {/each}
      </div>
    </fieldset>
    <TextArea id="continuing" label="Продолжающиеся эксперименты или восстановления" bind:value={continuingWork} />
    <TextArea id="debt-decision" label="Решение по оставшемуся долгу" bind:value={debtDecision} />
    <Button variant="primary" type="submit">Завершить изучение</Button>
  </form>
{/if}
