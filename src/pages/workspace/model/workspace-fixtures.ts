import type { Book, LibraryState } from "@/shared/api";

const fixedNow = 1_785_283_200;

export function emptyLibraryFixture(): LibraryState {
  return {
    books: [],
    drafts: [],
    ideas: [],
    topics: [],
    ideaLinks: [],
    experiments: [],
    experimentDrafts: [],
    recalls: [],
    materials: [],
    reviews: [],
    milestones: [],
    completionDrafts: [],
    workspaceNote: "",
    activeStudyBookId: null,
  };
}

export function activeLibraryFixture(): LibraryState {
  const state = emptyLibraryFixture();
  state.activeStudyBookId = "book-distributed";
  state.books = [
    book("book-distributed", "Designing Data-Intensive Applications", 286, false, false),
    book("book-domain", "Domain-Driven Design", 174, false, false),
    book("book-rust", "Programming Rust", 412, true, false),
    book("book-systems", "Systems Performance", 93, false, false),
    book("book-refactoring", "Refactoring", 221, true, true),
    book("book-platform", "Team Topologies", 78, false, false),
    book("book-testing", "Growing Object-Oriented Software", 156, false, false),
    book("book-architecture", "Software Architecture: The Hard Parts", 44, false, false),
    book("book-api", "A Philosophy of Software Design", 119, true, false),
  ];
  state.drafts = [
    {
      id: "draft-001",
      bookId: "book-distributed",
      section: "Глава 5 · Репликация",
      page: 286,
      excerpt: "The advantage of a leader-based approach is that conflict resolution happens on the leader.",
      context: "Однописательная модель упрощает порядок записей, но переносит отказоустойчивость в выбор лидера.",
      comment: "Сравнить с нашим журналом изменений.",
      fragments: [
        {
          page: 286,
          excerpt: "The advantage of a leader-based approach is that conflict resolution happens on the leader.",
          context: "Однописательная модель упрощает порядок записей.",
        },
      ],
      createdAt: fixedNow - 3_600,
    },
    {
      id: "draft-002",
      bookId: "book-domain",
      section: "Глава 4 · Изоляция домена",
      page: 174,
      excerpt: "A model is a selectively simplified and consciously structured form of knowledge.",
      context: "Модель отбрасывает детали, не нужные для решения текущей задачи.",
      comment: "",
      fragments: [
        {
          page: 174,
          excerpt: "A model is a selectively simplified and consciously structured form of knowledge.",
          context: "Модель отбрасывает ненужные детали.",
        },
      ],
      createdAt: fixedNow - 86_400,
    },
    {
      id: "draft-003",
      bookId: "book-distributed",
      section: "Глава 7 · Транзакции",
      page: 361,
      excerpt:
        "Transactions are an abstraction layer that allows an application to pretend certain concurrency problems do not exist.",
      context: "Граница страницы 360–361.",
      comment: "Проверить формулировку про абстракции отказов.",
      fragments: [
        { page: 360, excerpt: "Transactions are an abstraction layer", context: "Начало фрагмента" },
        { page: 361, excerpt: "certain concurrency problems do not exist", context: "Продолжение фрагмента" },
      ],
      createdAt: fixedNow - 172_800,
    },
  ];
  state.ideas = [
    {
      id: "idea-leader",
      bookId: "book-distributed",
      section: "Глава 5 · Репликация",
      formulation:
        "Единый лидер делает порядок записей понятным, но превращает смену лидера в часть доменного риска системы.",
      assignments: ["experiment", "recall"],
      fragments: [
        { page: 286, excerpt: "conflict resolution happens on the leader", context: "Leader-based replication" },
      ],
      versions: [
        {
          formulation: "Лидер упрощает разрешение конфликтов записи.",
          savedAt: fixedNow - 604_800,
        },
      ],
      topicIds: ["topic-reliability"],
    },
    {
      id: "idea-model",
      bookId: "book-domain",
      section: "Глава 4 · Изоляция домена",
      formulation: "Полезная модель намеренно неполна: её качество определяется решениями, которые она делает легче.",
      assignments: ["transfer"],
      fragments: [{ page: 174, excerpt: "selectively simplified", context: "What is a model?" }],
      versions: [],
      topicIds: ["topic-modeling"],
    },
    {
      id: "idea-quorum",
      bookId: "book-distributed",
      section: "Глава 5 · Репликация",
      formulation:
        "Кворум подтверждений задаёт компромисс между доступностью записи и уверенностью, что данные переживут отказ узлов.",
      assignments: ["recall"],
      fragments: [
        { page: 291, excerpt: "A write is durable after a quorum acknowledges it", context: "Quorum writes" },
      ],
      versions: [],
      topicIds: ["topic-reliability"],
    },
    {
      id: "idea-failure-model",
      bookId: "book-distributed",
      section: "Глава 8 · Проблемы распределённых систем",
      formulation:
        "Тайм-аут сообщает лишь об отсутствии ответа вовремя, поэтому неопределённость результата должна быть явной частью модели.",
      assignments: ["experiment"],
      fragments: [
        { page: 376, excerpt: "A timeout cannot tell whether the request succeeded", context: "Partial failures" },
      ],
      versions: [],
      topicIds: ["topic-reliability"],
    },
    {
      id: "idea-feedback",
      bookId: "book-refactoring",
      section: "Глава 2 · Принципы",
      formulation:
        "Маленькие преобразования ценны не размером, а короткой петлёй обратной связи после каждого изменения.",
      assignments: ["mastered"],
      fragments: [{ page: 47, excerpt: "small steps", context: "Refactoring workflow" }],
      versions: [],
      topicIds: ["topic-modeling"],
    },
  ];
  state.topics = [
    { id: "topic-reliability", name: "Надёжность распределённых систем" },
    { id: "topic-modeling", name: "Моделирование и обратная связь" },
  ];
  state.experiments = [
    {
      id: "experiment-001",
      ideaId: "idea-leader",
      situation: "Перепроектирование журнала конфигурации сервиса",
      action: "Сделать смену владельца журнала явным переходом состояния",
      result: "Команда увидела два ранее скрытых сценария восстановления",
      conclusion: "Лидерство полезно моделировать как аренду, а не как постоянную роль",
      status: "running",
      cancellationReason: "",
      nextStep: "Проверить восстановление после потери аренды",
    },
    {
      id: "experiment-002",
      ideaId: "idea-feedback",
      situation: "Сокращение цикла обратной связи при рефакторинге",
      action: "Выполнять по одному наблюдаемому преобразованию",
      result: "Отрицательный результат выявил слишком крупный первый шаг",
      conclusion: "Размер шага определяется скоростью проверки, а не числом строк",
      status: "completed",
      cancellationReason: "",
      nextStep: "",
    },
  ];
  state.recalls = [
    { id: "recall-001", ideaId: "idea-leader", answer: "", rating: "partial", nextAt: fixedNow + 172_800 },
  ];
  state.materials = [
    {
      id: "material-001",
      title: "Почему лидер — это риск, а не только роль",
      problem: "Команда считала failover инфраструктурной деталью.",
      idea: "Смена лидера меняет разрешённый порядок записи.",
      example: "Аренда владельца журнала.",
      result: "Сценарии отказа стали частью модели.",
      limitations: "Не относится к системам без единого писателя.",
      ideaIds: ["idea-leader"],
    },
  ];
  state.milestones = [
    {
      id: "milestone-read-239",
      bookId: "book-distributed",
      kind: "readingProgress",
      occurredAt: fixedNow - 518_400,
      page: 239,
    },
    { id: "milestone-read-286", bookId: "book-distributed", kind: "readingProgress", occurredAt: fixedNow, page: 286 },
    ...Array.from({ length: 3 }, (_, index) => ({
      id: `milestone-idea-${index}`,
      bookId: "book-distributed",
      kind: "ideaFormulated" as const,
      occurredAt: fixedNow - index * 86_400,
      page: null,
    })),
    ...Array.from({ length: 2 }, (_, index) => ({
      id: `milestone-recall-${index}`,
      bookId: "book-distributed",
      kind: "recallCompleted" as const,
      occurredAt: fixedNow - index * 86_400,
      page: null,
    })),
    {
      id: "milestone-experiment",
      bookId: "book-distributed",
      kind: "experimentAdvanced",
      occurredAt: fixedNow,
      page: null,
    },
    ...Array.from({ length: 7 }, (_, index) => ({
      id: `milestone-draft-${index}`,
      bookId: "book-distributed",
      kind: "draftResolved" as const,
      occurredAt: fixedNow - index * 86_400,
      page: null,
    })),
  ];
  return state;
}

function book(id: string, title: string, page: number, readingCompleted: boolean, studyCompleted: boolean): Book {
  return {
    id,
    title,
    storedFile: `${id}.pdf`,
    hasTextLayer: id !== "book-systems",
    outline: [
      { id: `${id}-outline-1`, title: "Введение", page: 1, parentId: null },
      { id: `${id}-outline-2`, title: "Модели и границы", page: Math.max(24, page - 96), parentId: null },
      { id: `${id}-outline-3`, title: "Надёжность на практике", page, parentId: null },
    ],
    reading: { page, zoom: 1.15, scroll: 0.32 },
    farthestPage: Math.max(page, Math.min(page + 26, 438)),
    pageCount: 612,
    contentHash: `fixture-${id}`,
    reader: {
      documentMode: "mutedLight",
      invertImages: true,
      sidebarOpen: false,
      sidebarTab: "note",
      sidebarWidth: 400,
    },
    studyStatus: studyCompleted ? "completed" : id === "book-distributed" ? "active" : "paused",
    studyCycles: [],
    archived: false,
    readingCompleted,
    retrospective: studyCompleted
      ? {
          text: "Книга изменила способ дробить преобразования и проверять результат.",
          significantIdeaIds: [],
          continuingWork: "Продолжить наблюдать длительность обратной связи.",
          unfinishedWorkDecision: "Незавершённые эксперименты продолжаются отдельно.",
          workDecisions: [],
        }
      : null,
  };
}
