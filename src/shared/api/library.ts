import { invoke } from "@tauri-apps/api/core";

export type OutlineItem = { id: string; title: string; page: number; parentId?: string | null };
export type ReadingPosition = { page: number; zoom: number; scroll: number };
export type Book = {
  id: string;
  title: string;
  storedFile: string;
  hasTextLayer: boolean;
  outline: OutlineItem[];
  reading: ReadingPosition;
  readingCompleted: boolean;
  studyCompleted: boolean;
  retrospective?: Retrospective | null;
};
export type DraftNote = { id: string; bookId: string; section: string; page: number; excerpt: string; context: string; comment: string; createdAt: number };
export type SourceFragment = { page: number; excerpt: string; context: string };
export type IdeaVersion = { formulation: string; savedAt: number };
export type Idea = { id: string; bookId: string; section: string; formulation: string; assignments: string[]; fragments: SourceFragment[]; versions: IdeaVersion[]; topicIds: string[] };
export type Topic = { id: string; name: string };
export type IdeaLink = { id: string; fromIdeaId: string; toIdeaId: string; relation: string };
export type Experiment = { id: string; ideaId: string; situation: string; action: string; result: string; conclusion: string; successful: boolean; completed: boolean };
export type Recall = { id: string; ideaId: string; answer: string; rating: string; nextAt: number };
export type StudySession = { id: string; intention: string; plannedAt: number; status: string; resolutionReason: string };
export type TransferMaterial = { id: string; title: string; problem: string; idea: string; example: string; result: string; limitations: string; ideaIds: string[] };
export type IdeaReview = { id: string; ideaId: string; decision: string; conclusion: string; pending: boolean };
export type Retrospective = { text: string; significantIdeaIds: string[]; continuingWork: string; debtDecision: string };

export type LibraryState = {
  books: Book[];
  drafts: DraftNote[];
  ideas: Idea[];
  topics: Topic[];
  ideaLinks: IdeaLink[];
  experiments: Experiment[];
  recalls: Recall[];
  sessions: StudySession[];
  materials: TransferMaterial[];
  reviews: IdeaReview[];
  workspaceNote: string;
  activeStudyBookId?: string | null;
  weeklySessionBudget: number;
  lastDebtChange: number;
  lastDebtChangedAt: number;
  debtNotificationSentAt?: number | null;
};

export type LibraryAction =
  | { kind: "saveWorkspaceNote"; note: string }
  | { kind: "updateReading"; bookId: string; page: number; zoom: number; scroll: number }
  | { kind: "saveOutline"; bookId: string; outline: OutlineItem[] }
  | { kind: "captureDraft"; bookId: string; section: string; page: number; excerpt: string; context: string; comment: string }
  | { kind: "resolveDraftAsIdea"; draftId: string; formulation: string; section: string; assignments: string[] }
  | { kind: "attachDraftToIdea"; draftId: string; ideaId: string }
  | { kind: "discardDraft"; draftId: string }
  | { kind: "activateStudy"; bookId: string }
  | { kind: "completeReading"; bookId: string }
  | { kind: "setStudyRhythm"; weeklySessionBudget: number }
  | { kind: "planSession"; intention: string; plannedAt: number }
  | { kind: "resolveSession"; sessionId: string; status: string; reason: string }
  | { kind: "updateIdea"; ideaId: string; formulation: string; assignments: string[] }
  | { kind: "createTopic"; name: string }
  | { kind: "assignTopic"; ideaId: string; topicId: string }
  | { kind: "linkIdeas"; fromIdeaId: string; toIdeaId: string; relation: string }
  | { kind: "completeExperiment"; ideaId: string; situation: string; action: string; result: string; conclusion: string; successful: boolean }
  | { kind: "completeRecall"; ideaId: string; answer: string; rating: string; nextAt?: number | null }
  | { kind: "saveMaterial"; title: string; problem: string; idea: string; example: string; result: string; limitations: string; ideaIds: string[] }
  | { kind: "resolveReview"; ideaId: string; decision: string; formulation: string; conclusion: string }
  | { kind: "completeStudy"; bookId: string; retrospective: string; significantIdeaIds: string[]; continuingWork: string; debtDecision: string };

export type SearchResult = { id: string; kind: "book" | "idea"; title: string; context: string };
type CommandError = { code?: string; message?: string };

const emptyFields: Omit<LibraryState, "books" | "workspaceNote"> = {
  drafts: [], ideas: [], topics: [], ideaLinks: [], experiments: [], recalls: [], sessions: [], materials: [], reviews: [],
  activeStudyBookId: null, weeklySessionBudget: 3, lastDebtChange: 0, lastDebtChangedAt: 0, debtNotificationSentAt: null,
};

function normalizeLibrary(value: Partial<LibraryState>): LibraryState {
  return { ...emptyFields, books: [], workspaceNote: "", ...value };
}

export async function loadLibrary(): Promise<LibraryState> { return normalizeLibrary(await invoke<LibraryState>("load_library")); }
export async function saveWorkspaceNote(note: string): Promise<LibraryState> { return normalizeLibrary(await invoke<LibraryState>("save_workspace_note", { note })); }
export async function executeLibraryAction(action: LibraryAction): Promise<LibraryState> { return normalizeLibrary(await invoke<LibraryState>("execute_library_action", { action })); }
export async function importPdf(path: string, title = ""): Promise<LibraryState> { return normalizeLibrary(await invoke<LibraryState>("import_pdf", { path, title })); }
export async function searchLibrary(query: string): Promise<SearchResult[]> { return invoke<SearchResult[]>("search_library", { query }); }
export async function bookFilePath(bookId: string): Promise<string> { return invoke<string>("book_file_path", { bookId }); }
export async function exportLibraryArchive(path: string, password: string): Promise<void> { return invoke("export_library_archive", { path, password }); }
export async function importLibraryArchive(path: string, password: string): Promise<LibraryState> { return normalizeLibrary(await invoke<LibraryState>("import_library_archive", { path, password })); }
export async function restoreLatestSnapshot(): Promise<LibraryState> { return normalizeLibrary(await invoke<LibraryState>("restore_latest_snapshot")); }
export async function exportMaterialMarkdown(materialId: string, path: string): Promise<void> { return invoke("export_material_markdown", { materialId, path }); }
export async function exportDraftMarkdown(draftId: string, path: string): Promise<LibraryState> { return normalizeLibrary(await invoke<LibraryState>("export_draft_markdown", { draftId, path })); }
export async function installSignedUpdate(): Promise<boolean> { return invoke<boolean>("install_signed_update"); }

export function commandErrorMessage(cause: unknown): string {
  if (typeof cause === "object" && cause !== null) {
    const error = cause as CommandError;
    if (typeof error.message === "string") return error.message;
  }
  return String(cause);
}
