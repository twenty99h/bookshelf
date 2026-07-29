use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Write},
    iter,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LibraryState {
    pub books: Vec<Book>,
    pub drafts: Vec<DraftNote>,
    pub ideas: Vec<Idea>,
    pub topics: Vec<Topic>,
    pub idea_links: Vec<IdeaLink>,
    pub experiments: Vec<Experiment>,
    pub recalls: Vec<Recall>,
    pub sessions: Vec<StudySession>,
    pub materials: Vec<TransferMaterial>,
    pub reviews: Vec<IdeaReview>,
    pub workspace_note: String,
    pub active_study_book_id: Option<String>,
    pub weekly_session_budget: u8,
    pub last_debt_change: i32,
    pub last_debt_changed_at: u64,
    pub debt_notification_sent_at: Option<u64>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Book {
    pub id: String,
    pub title: String,
    pub stored_file: String,
    pub has_text_layer: bool,
    pub outline: Vec<OutlineItem>,
    pub reading: ReadingPosition,
    pub reading_completed: bool,
    pub study_completed: bool,
    pub retrospective: Option<Retrospective>,
}

impl Book {
    #[cfg(test)]
    fn for_test(id: &str, title: &str) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            has_text_layer: true,
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutlineItem {
    pub id: String,
    pub title: String,
    pub page: u32,
    pub parent_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadingPosition {
    pub page: u32,
    pub zoom: f32,
    pub scroll: f32,
}

impl Default for ReadingPosition {
    fn default() -> Self {
        Self {
            page: 1,
            zoom: 1.0,
            scroll: 0.0,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftNote {
    pub id: String,
    pub book_id: String,
    pub section: String,
    pub page: u32,
    pub excerpt: String,
    pub context: String,
    pub comment: String,
    pub created_at: u64,
}

impl DraftNote {
    #[cfg(test)]
    fn for_test(id: &str, book_id: &str) -> Self {
        Self {
            id: id.into(),
            book_id: book_id.into(),
            section: "Глава 1".into(),
            excerpt: "Фрагмент".into(),
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Idea {
    pub id: String,
    pub book_id: String,
    pub section: String,
    pub formulation: String,
    pub assignments: Vec<String>,
    pub fragments: Vec<SourceFragment>,
    pub versions: Vec<IdeaVersion>,
    pub topic_ids: Vec<String>,
}

impl Idea {
    #[cfg(test)]
    fn for_test(id: &str, book_id: &str) -> Self {
        Self {
            id: id.into(),
            book_id: book_id.into(),
            section: "Глава 1".into(),
            formulation: "Идея".into(),
            assignments: vec!["recall".into()],
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceFragment {
    pub page: u32,
    pub excerpt: String,
    pub context: String,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IdeaVersion {
    pub formulation: String,
    pub saved_at: u64,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Topic {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IdeaLink {
    pub id: String,
    pub from_idea_id: String,
    pub to_idea_id: String,
    pub relation: String,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Experiment {
    pub id: String,
    pub idea_id: String,
    pub situation: String,
    pub action: String,
    pub result: String,
    pub conclusion: String,
    pub successful: bool,
    pub completed: bool,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Recall {
    pub id: String,
    pub idea_id: String,
    pub answer: String,
    pub rating: String,
    pub next_at: u64,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudySession {
    pub id: String,
    pub intention: String,
    pub planned_at: u64,
    pub status: String,
    pub resolution_reason: String,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferMaterial {
    pub id: String,
    pub title: String,
    pub problem: String,
    pub idea: String,
    pub example: String,
    pub result: String,
    pub limitations: String,
    pub idea_ids: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IdeaReview {
    pub id: String,
    pub idea_id: String,
    pub decision: String,
    pub conclusion: String,
    pub pending: bool,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Retrospective {
    pub text: String,
    pub significant_idea_ids: Vec<String>,
    pub continuing_work: String,
    pub debt_decision: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum LibraryAction {
    SaveWorkspaceNote {
        note: String,
    },
    UpdateReading {
        book_id: String,
        page: u32,
        zoom: f32,
        scroll: f32,
    },
    SaveOutline {
        book_id: String,
        outline: Vec<OutlineItem>,
    },
    CaptureDraft {
        book_id: String,
        section: String,
        page: u32,
        excerpt: String,
        context: String,
        comment: String,
    },
    ResolveDraftAsIdea {
        draft_id: String,
        formulation: String,
        section: String,
        assignments: Vec<String>,
    },
    AttachDraftToIdea {
        draft_id: String,
        idea_id: String,
    },
    DiscardDraft {
        draft_id: String,
    },
    ActivateStudy {
        book_id: String,
    },
    CompleteReading {
        book_id: String,
    },
    SetStudyRhythm {
        weekly_session_budget: u8,
    },
    PlanSession {
        intention: String,
        planned_at: u64,
    },
    ResolveSession {
        session_id: String,
        status: String,
        reason: String,
    },
    UpdateIdea {
        idea_id: String,
        formulation: String,
        assignments: Vec<String>,
    },
    CreateTopic {
        name: String,
    },
    AssignTopic {
        idea_id: String,
        topic_id: String,
    },
    LinkIdeas {
        from_idea_id: String,
        to_idea_id: String,
        relation: String,
    },
    CompleteExperiment {
        idea_id: String,
        situation: String,
        action: String,
        result: String,
        conclusion: String,
        successful: bool,
    },
    CompleteRecall {
        idea_id: String,
        answer: String,
        rating: String,
        next_at: Option<u64>,
    },
    SaveMaterial {
        title: String,
        problem: String,
        idea: String,
        example: String,
        result: String,
        limitations: String,
        idea_ids: Vec<String>,
    },
    ResolveReview {
        idea_id: String,
        decision: String,
        formulation: String,
        conclusion: String,
    },
    CompleteStudy {
        book_id: String,
        retrospective: String,
        significant_idea_ids: Vec<String>,
        continuing_work: String,
        debt_decision: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct DomainError {
    pub code: &'static str,
    pub message: String,
}

impl DomainError {
    fn new(code: &'static str, message: &str) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl LibraryState {
    pub fn debt(&self) -> usize {
        self.drafts.len()
            + self
                .experiments
                .iter()
                .filter(|item| !item.completed)
                .count()
            + self.reviews.iter().filter(|item| item.pending).count()
    }

    pub fn apply(&mut self, action: LibraryAction) -> Result<(), DomainError> {
        match action {
            LibraryAction::SaveWorkspaceNote { note } => {
                if note.chars().count() > 240 {
                    return Err(DomainError::new(
                        "workspace_note_too_long",
                        "Пометка не может быть длиннее 240 символов",
                    ));
                }
                self.workspace_note = note;
            }
            LibraryAction::UpdateReading {
                book_id,
                page,
                zoom,
                scroll,
            } => {
                let book = find_book_mut(self, &book_id)?;
                if page == 0 || !(0.5..=4.0).contains(&zoom) {
                    return Err(DomainError::new(
                        "reading_position_invalid",
                        "Проверьте страницу и масштаб",
                    ));
                }
                book.reading = ReadingPosition {
                    page,
                    zoom,
                    scroll: scroll.max(0.0),
                };
            }
            LibraryAction::SaveOutline { book_id, outline } => {
                if outline
                    .iter()
                    .any(|item| item.title.trim().is_empty() || item.page == 0)
                {
                    return Err(DomainError::new(
                        "outline_item_invalid",
                        "Укажите название и страницу раздела",
                    ));
                }
                find_book_mut(self, &book_id)?.outline = outline;
            }
            LibraryAction::CaptureDraft {
                book_id,
                section,
                page,
                excerpt,
                context,
                comment,
            } => {
                find_book(self, &book_id)?;
                if excerpt.trim().is_empty() || page == 0 {
                    return Err(DomainError::new(
                        "draft_source_required",
                        "Выберите фрагмент и страницу",
                    ));
                }
                self.drafts.push(DraftNote {
                    id: new_id("draft"),
                    book_id,
                    section,
                    page,
                    excerpt,
                    context,
                    comment,
                    created_at: now(),
                });
            }
            LibraryAction::ResolveDraftAsIdea {
                draft_id,
                formulation,
                section,
                assignments,
            } => {
                if formulation.trim().is_empty()
                    || section.trim().is_empty()
                    || assignments.is_empty()
                {
                    return Err(DomainError::new(
                        "idea_fields_required",
                        "Нужны авторская формулировка, раздел и хотя бы одно назначение",
                    ));
                }
                let draft = self
                    .drafts
                    .iter()
                    .find(|item| item.id == draft_id)
                    .cloned()
                    .ok_or_else(|| {
                        DomainError::new("draft_not_found", "Черновая заметка не найдена")
                    })?;
                let idea = Idea {
                    id: new_id("idea"),
                    book_id: draft.book_id,
                    section,
                    formulation: formulation.clone(),
                    assignments,
                    fragments: vec![SourceFragment {
                        page: draft.page,
                        excerpt: draft.excerpt,
                        context: draft.context,
                    }],
                    versions: vec![IdeaVersion {
                        formulation,
                        saved_at: now(),
                    }],
                    topic_ids: vec![],
                };
                self.ideas.push(idea);
                self.drafts.retain(|item| item.id != draft_id);
            }
            LibraryAction::AttachDraftToIdea { draft_id, idea_id } => {
                let draft = self
                    .drafts
                    .iter()
                    .find(|item| item.id == draft_id)
                    .cloned()
                    .ok_or_else(|| {
                        DomainError::new("draft_not_found", "Черновая заметка не найдена")
                    })?;
                let idea = find_idea_mut(self, &idea_id)?;
                idea.fragments.push(SourceFragment {
                    page: draft.page,
                    excerpt: draft.excerpt,
                    context: draft.context,
                });
                self.drafts.retain(|item| item.id != draft_id);
            }
            LibraryAction::DiscardDraft { draft_id } => {
                if !self.drafts.iter().any(|item| item.id == draft_id) {
                    return Err(DomainError::new(
                        "draft_not_found",
                        "Черновая заметка не найдена",
                    ));
                }
                self.drafts.retain(|item| item.id != draft_id);
            }
            LibraryAction::ActivateStudy { book_id } => {
                find_book(self, &book_id)?;
                self.active_study_book_id = Some(book_id);
            }
            LibraryAction::CompleteReading { book_id } => {
                find_book_mut(self, &book_id)?.reading_completed = true
            }
            LibraryAction::SetStudyRhythm {
                weekly_session_budget,
            } => {
                if !(1..=14).contains(&weekly_session_budget) {
                    return Err(DomainError::new(
                        "study_rhythm_invalid",
                        "Выберите от 1 до 14 сеансов в неделю",
                    ));
                }
                self.weekly_session_budget = weekly_session_budget;
            }
            LibraryAction::PlanSession {
                intention,
                planned_at,
            } => {
                if intention.trim().is_empty() {
                    return Err(DomainError::new(
                        "session_intention_required",
                        "Опишите намерение сеанса",
                    ));
                }
                self.sessions.push(StudySession {
                    id: new_id("session"),
                    intention,
                    planned_at,
                    status: "planned".into(),
                    resolution_reason: String::new(),
                });
            }
            LibraryAction::ResolveSession {
                session_id,
                status,
                reason,
            } => {
                if !["completed", "moved", "replaced", "cancelled"].contains(&status.as_str()) {
                    return Err(DomainError::new(
                        "session_resolution_invalid",
                        "Выберите результат сеанса",
                    ));
                }
                if status != "completed" && reason.trim().is_empty() {
                    return Err(DomainError::new(
                        "session_reason_required",
                        "Укажите причину решения",
                    ));
                }
                let session = self
                    .sessions
                    .iter_mut()
                    .find(|item| item.id == session_id)
                    .ok_or_else(|| DomainError::new("session_not_found", "Сеанс не найден"))?;
                session.status = status;
                session.resolution_reason = reason;
            }
            LibraryAction::UpdateIdea {
                idea_id,
                formulation,
                assignments,
            } => {
                if formulation.trim().is_empty() || assignments.is_empty() {
                    return Err(DomainError::new(
                        "idea_fields_required",
                        "Нужны формулировка и назначение",
                    ));
                }
                let idea = find_idea_mut(self, &idea_id)?;
                if idea.formulation != formulation {
                    idea.versions.push(IdeaVersion {
                        formulation: formulation.clone(),
                        saved_at: now(),
                    });
                }
                idea.formulation = formulation;
                idea.assignments = assignments;
            }
            LibraryAction::CreateTopic { name } => {
                if name.trim().is_empty() {
                    return Err(DomainError::new(
                        "topic_name_required",
                        "Назовите тему знаний",
                    ));
                }
                self.topics.push(Topic {
                    id: new_id("topic"),
                    name,
                });
            }
            LibraryAction::AssignTopic { idea_id, topic_id } => {
                if !self.topics.iter().any(|item| item.id == topic_id) {
                    return Err(DomainError::new("topic_not_found", "Тема не найдена"));
                }
                let idea = find_idea_mut(self, &idea_id)?;
                if !idea.topic_ids.contains(&topic_id) {
                    idea.topic_ids.push(topic_id);
                }
            }
            LibraryAction::LinkIdeas {
                from_idea_id,
                to_idea_id,
                relation,
            } => {
                if from_idea_id == to_idea_id
                    || !["complements", "clarifies", "contradicts"].contains(&relation.as_str())
                {
                    return Err(DomainError::new(
                        "idea_link_invalid",
                        "Выберите две идеи и допустимый тип связи",
                    ));
                }
                find_idea(self, &from_idea_id)?;
                find_idea(self, &to_idea_id)?;
                self.idea_links.push(IdeaLink {
                    id: new_id("link"),
                    from_idea_id,
                    to_idea_id,
                    relation,
                });
            }
            LibraryAction::CompleteExperiment {
                idea_id,
                situation,
                action,
                result,
                conclusion,
                successful,
            } => {
                find_idea(self, &idea_id)?;
                if [
                    situation.as_str(),
                    action.as_str(),
                    result.as_str(),
                    conclusion.as_str(),
                ]
                .iter()
                .any(|value| value.trim().is_empty())
                {
                    return Err(DomainError::new(
                        "experiment_fields_required",
                        "Заполните ситуацию, действие, результат и вывод",
                    ));
                }
                self.experiments.push(Experiment {
                    id: new_id("experiment"),
                    idea_id,
                    situation,
                    action,
                    result,
                    conclusion,
                    successful,
                    completed: true,
                });
            }
            LibraryAction::CompleteRecall {
                idea_id,
                answer,
                rating,
                next_at,
            } => {
                find_idea(self, &idea_id)?;
                if answer.trim().is_empty()
                    || !["confident", "partial", "missed"].contains(&rating.as_str())
                {
                    return Err(DomainError::new(
                        "recall_invalid",
                        "Напишите ответ и выберите самооценку",
                    ));
                }
                let suggested_days = match rating.as_str() {
                    "confident" => 30,
                    "partial" => 7,
                    _ => 1,
                };
                self.recalls.push(Recall {
                    id: new_id("recall"),
                    idea_id,
                    answer,
                    rating,
                    next_at: next_at.unwrap_or_else(|| now() + suggested_days * 86_400),
                });
            }
            LibraryAction::SaveMaterial {
                title,
                problem,
                idea,
                example,
                result,
                limitations,
                idea_ids,
            } => {
                if title.trim().is_empty() || idea_ids.is_empty() {
                    return Err(DomainError::new(
                        "material_fields_required",
                        "Назовите материал и выберите идеи",
                    ));
                }
                self.materials.push(TransferMaterial {
                    id: new_id("material"),
                    title,
                    problem,
                    idea,
                    example,
                    result,
                    limitations,
                    idea_ids,
                });
            }
            LibraryAction::ResolveReview {
                idea_id,
                decision,
                formulation,
                conclusion,
            } => {
                find_idea(self, &idea_id)?;
                if !["refined", "unchanged", "later"].contains(&decision.as_str()) {
                    return Err(DomainError::new(
                        "review_decision_invalid",
                        "Выберите решение по проверке",
                    ));
                }
                if decision == "refined" {
                    self.apply(LibraryAction::UpdateIdea {
                        idea_id: idea_id.clone(),
                        formulation,
                        assignments: find_idea(self, &idea_id)?.assignments.clone(),
                    })?;
                }
                self.reviews.retain(|item| item.idea_id != idea_id);
                self.reviews.push(IdeaReview {
                    id: new_id("review"),
                    idea_id,
                    decision: decision.clone(),
                    conclusion,
                    pending: decision == "later",
                });
            }
            LibraryAction::CompleteStudy {
                book_id,
                retrospective,
                significant_idea_ids,
                continuing_work,
                debt_decision,
            } => {
                find_book(self, &book_id)?;
                if retrospective.trim().is_empty()
                    || !(3..=7).contains(&significant_idea_ids.len())
                    || debt_decision.trim().is_empty()
                {
                    return Err(DomainError::new(
                        "retrospective_required",
                        "Добавьте ретроспективу, 3–7 значимых идей и решение по долгу",
                    ));
                }
                if significant_idea_ids.iter().any(|id| {
                    !self
                        .ideas
                        .iter()
                        .any(|idea| &idea.id == id && idea.book_id == book_id)
                }) {
                    return Err(DomainError::new(
                        "retrospective_ideas_invalid",
                        "Значимые идеи должны относиться к этой книге",
                    ));
                }
                let book = find_book_mut(self, &book_id)?;
                book.study_completed = true;
                book.retrospective = Some(Retrospective {
                    text: retrospective,
                    significant_idea_ids,
                    continuing_work,
                    debt_decision,
                });
                if self.active_study_book_id.as_deref() == Some(book_id.as_str()) {
                    self.active_study_book_id = None;
                }
            }
        }
        Ok(())
    }
}

fn find_book<'a>(state: &'a LibraryState, id: &str) -> Result<&'a Book, DomainError> {
    state
        .books
        .iter()
        .find(|item| item.id == id)
        .ok_or_else(|| DomainError::new("book_not_found", "Книга не найдена"))
}
fn find_book_mut<'a>(state: &'a mut LibraryState, id: &str) -> Result<&'a mut Book, DomainError> {
    state
        .books
        .iter_mut()
        .find(|item| item.id == id)
        .ok_or_else(|| DomainError::new("book_not_found", "Книга не найдена"))
}
fn find_idea<'a>(state: &'a LibraryState, id: &str) -> Result<&'a Idea, DomainError> {
    state
        .ideas
        .iter()
        .find(|item| item.id == id)
        .ok_or_else(|| DomainError::new("idea_not_found", "Идея не найдена"))
}
fn find_idea_mut<'a>(state: &'a mut LibraryState, id: &str) -> Result<&'a mut Idea, DomainError> {
    state
        .ideas
        .iter_mut()
        .find(|item| item.id == id)
        .ok_or_else(|| DomainError::new("idea_not_found", "Идея не найдена"))
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn unique_number() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        + NEXT_ID.fetch_add(1, Ordering::Relaxed) as u128
}
fn new_id(prefix: &str) -> String {
    format!("{prefix}-{}", unique_number())
}

pub struct Library {
    data_dir: PathBuf,
    database_file: PathBuf,
}

#[derive(Deserialize, Serialize)]
struct ArchiveManifest {
    version: u32,
    state: LibraryState,
}

impl Library {
    pub fn open(data_dir: impl AsRef<Path>) -> io::Result<Self> {
        fs::create_dir_all(data_dir.as_ref())?;
        fs::create_dir_all(data_dir.as_ref().join("books"))?;
        fs::create_dir_all(data_dir.as_ref().join("snapshots"))?;
        let library = Self {
            data_dir: data_dir.as_ref().into(),
            database_file: data_dir.as_ref().join("library.sqlite3"),
        };
        library.migrate()?;
        Ok(library)
    }

    pub fn load(&self) -> io::Result<LibraryState> {
        let connection = Connection::open(&self.database_file).map_err(sqlite_io)?;
        let json: Option<String> = connection
            .query_row("SELECT json FROM workspace_state WHERE id = 1", [], |row| {
                row.get(0)
            })
            .optional()
            .map_err(sqlite_io)?;
        json.map(|value| serde_json::from_str(&value).map_err(io::Error::other))
            .unwrap_or_else(|| Ok(LibraryState::default()))
    }

    pub fn absolute_book_path(&self, stored_file: &str) -> PathBuf {
        self.data_dir.join(stored_file)
    }

    pub fn apply(&self, action: LibraryAction) -> Result<LibraryState, LibraryError> {
        let mut state = self.load()?;
        let previous_debt = state.debt() as i32;
        state.apply(action)?;
        state.last_debt_change = state.debt() as i32 - previous_debt;
        if state.last_debt_change != 0 {
            state.last_debt_changed_at = now();
            state.debt_notification_sent_at = None;
        }
        self.replace_state(&state)?;
        self.create_snapshot(&state)?;
        Ok(state)
    }

    pub fn save_workspace_note(&self, note: String) -> io::Result<LibraryState> {
        self.apply(LibraryAction::SaveWorkspaceNote { note })
            .map_err(io::Error::other)
    }

    pub fn claim_debt_notification(
        &self,
        quiet_period_seconds: u64,
    ) -> Result<Option<usize>, LibraryError> {
        let mut state = self.load()?;
        let due = state.debt() > 0
            && state.last_debt_changed_at > 0
            && now().saturating_sub(state.last_debt_changed_at) >= quiet_period_seconds
            && state.debt_notification_sent_at.is_none();
        if !due {
            return Ok(None);
        }
        let debt = state.debt();
        state.debt_notification_sent_at = Some(now());
        self.replace_state(&state)?;
        Ok(Some(debt))
    }

    pub fn import_pdf(
        &self,
        source: impl AsRef<Path>,
        title: String,
    ) -> Result<LibraryState, LibraryError> {
        let bytes = fs::read(source.as_ref())?;
        if !bytes.starts_with(b"%PDF") {
            return Err(DomainError::new("pdf_invalid", "Выбранный файл не является PDF").into());
        }
        let has_text_layer = bytes.windows(3).any(|part| part == b" BT")
            || bytes.windows(5).any(|part| part == b"/Font");
        if !has_text_layer {
            return Err(DomainError::new(
                "pdf_text_layer_missing",
                "В PDF нет пригодного текстового слоя. OCR пока не поддерживается",
            )
            .into());
        }
        let title = if title.trim().is_empty() {
            source
                .as_ref()
                .file_stem()
                .and_then(|item| item.to_str())
                .unwrap_or("Книга")
                .to_owned()
        } else {
            title
        };
        let id = new_id("book");
        let relative = format!("books/{id}.pdf");
        let target = self.data_dir.join(&relative);
        let temporary = target.with_extension("pdf.tmp");
        fs::write(&temporary, bytes)?;
        fs::rename(&temporary, &target)?;
        let mut state = self.load()?;
        state.books.push(Book {
            id,
            title,
            stored_file: relative,
            has_text_layer,
            ..Book::default()
        });
        if let Err(error) = self.replace_state(&state) {
            let _ = fs::remove_file(&target);
            return Err(error.into());
        }
        self.create_snapshot(&state)?;
        Ok(state)
    }

    pub fn search(&self, query: &str) -> io::Result<Vec<SearchResult>> {
        let terms = query
            .split_whitespace()
            .map(|term| format!("\"{}\"", term.replace('"', "\"\"")))
            .collect::<Vec<_>>()
            .join(" AND ");
        if terms.is_empty() {
            return Ok(vec![]);
        }
        let connection = Connection::open(&self.database_file).map_err(sqlite_io)?;
        let mut statement = connection.prepare("SELECT entity_id, kind, title, context FROM search_index WHERE search_index MATCH ?1 ORDER BY rank LIMIT 50").map_err(sqlite_io)?;
        let rows = statement
            .query_map([terms], |row| {
                Ok(SearchResult {
                    id: row.get(0)?,
                    kind: row.get(1)?,
                    title: row.get(2)?,
                    context: row.get(3)?,
                })
            })
            .map_err(sqlite_io)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(sqlite_io)
    }

    pub fn export_archive(
        &self,
        destination: impl AsRef<Path>,
        password: &str,
    ) -> Result<(), LibraryError> {
        if password.chars().count() < 8 {
            return Err(DomainError::new(
                "archive_password_weak",
                "Пароль архива должен содержать не менее 8 символов",
            )
            .into());
        }
        let temporary = destination.as_ref().with_extension("age.tmp");
        remove_if_present(&temporary)?;
        let output = fs::File::create(&temporary)?;
        let passphrase = age::secrecy::SecretString::from(password.to_owned());
        let encryptor = age::Encryptor::with_user_passphrase(passphrase);
        let encrypted = encryptor.wrap_output(output).map_err(crypto_io)?;
        let mut archive = tar::Builder::new(encrypted);
        let manifest = ArchiveManifest {
            version: 1,
            state: self.load()?,
        };
        append_bytes(
            &mut archive,
            "manifest.json",
            &serde_json::to_vec(&manifest).map_err(io::Error::other)?,
        )?;
        for book in &manifest.state.books {
            let source = self.absolute_book_path(&book.stored_file);
            if source.exists() {
                archive.append_path_with_name(source, &book.stored_file)?;
            }
        }
        let encrypted = archive.into_inner()?;
        encrypted.finish().map_err(crypto_io)?;
        fs::rename(&temporary, destination.as_ref())?;
        Ok(())
    }

    pub fn import_archive(
        &self,
        source: impl AsRef<Path>,
        password: &str,
    ) -> Result<LibraryState, LibraryError> {
        let input = fs::File::open(source)?;
        let decryptor = age::Decryptor::new(input).map_err(|_| {
            DomainError::new(
                "archive_corrupt",
                "Архив повреждён или имеет неизвестный формат",
            )
        })?;
        let passphrase = age::secrecy::SecretString::from(password.to_owned());
        let identity = age::scrypt::Identity::new(passphrase);
        let reader = decryptor
            .decrypt(iter::once(&identity as &dyn age::Identity))
            .map_err(|_| DomainError::new("archive_password_invalid", "Неверный пароль архива"))?;
        let staging = tempfile::tempdir()?;
        let mut archive = tar::Archive::new(reader);
        archive.unpack(staging.path()).map_err(|_| {
            DomainError::new("archive_corrupt", "Не удалось проверить целостность архива")
        })?;
        let manifest_bytes = fs::read(staging.path().join("manifest.json")).map_err(|_| {
            DomainError::new(
                "archive_corrupt",
                "В архиве отсутствует описание библиотеки",
            )
        })?;
        let manifest: ArchiveManifest = serde_json::from_slice(&manifest_bytes)
            .map_err(|_| DomainError::new("archive_corrupt", "Описание библиотеки повреждено"))?;
        if manifest.version != 1 {
            return Err(DomainError::new(
                "archive_version_unsupported",
                "Версия архива не поддерживается",
            )
            .into());
        }
        let current = self.load()?;
        if current != LibraryState::default() {
            return Err(DomainError::new(
                "archive_target_not_empty",
                "Импорт возможен только в пустую библиотеку: сохраните текущую библиотеку отдельно или используйте чистую установку",
            )
            .into());
        }
        for book in &manifest.state.books {
            let staged = staging.path().join(&book.stored_file);
            if !staged.is_file() {
                return Err(
                    DomainError::new("archive_corrupt", "В архиве отсутствует файл книги").into(),
                );
            }
        }
        let mut prepared: Vec<(PathBuf, PathBuf)> = Vec::new();
        for book in &manifest.state.books {
            let target = self.absolute_book_path(&book.stored_file);
            if target.exists() {
                cleanup_paths(prepared.iter().map(|(temporary, _)| temporary));
                return Err(DomainError::new(
                    "archive_duplicates",
                    "Файл этой книги уже существует. Удалите оставшиеся данные или выберите чистую библиотеку",
                )
                .into());
            }
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            let temporary = target.with_extension(format!("pdf.importing-{}", unique_number()));
            if let Err(error) = fs::copy(staging.path().join(&book.stored_file), &temporary) {
                cleanup_paths(prepared.iter().map(|(path, _)| path));
                return Err(error.into());
            }
            prepared.push((temporary, target));
        }
        let mut committed: Vec<PathBuf> = Vec::new();
        for (temporary, target) in &prepared {
            if let Err(error) = fs::rename(temporary, target) {
                cleanup_paths(prepared.iter().map(|(path, _)| path));
                cleanup_paths(committed.iter());
                return Err(error.into());
            }
            committed.push(target.clone());
        }
        if let Err(error) = self.replace_state(&manifest.state) {
            cleanup_paths(committed.iter());
            return Err(error.into());
        }
        self.create_snapshot(&manifest.state)?;
        Ok(manifest.state)
    }

    pub fn restore_latest_snapshot(&self) -> Result<LibraryState, LibraryError> {
        let dir = self.data_dir.join("snapshots");
        let mut snapshots: Vec<_> = fs::read_dir(dir)?.filter_map(Result::ok).collect();
        snapshots.sort_by_key(|entry| entry.file_name());
        let latest = snapshots.last().ok_or_else(|| {
            DomainError::new(
                "snapshot_not_found",
                "Нет доступного снимка рабочего состояния",
            )
        })?;
        let state: LibraryState =
            serde_json::from_slice(&fs::read(latest.path())?).map_err(|_| {
                DomainError::new("snapshot_corrupt", "Снимок рабочего состояния повреждён")
            })?;
        let valid_sources = state
            .drafts
            .iter()
            .all(|draft| state.books.iter().any(|book| book.id == draft.book_id))
            && state
                .ideas
                .iter()
                .all(|idea| state.books.iter().any(|book| book.id == idea.book_id));
        if !valid_sources {
            return Err(DomainError::new(
                "snapshot_inconsistent",
                "Снимок содержит несогласованные источники",
            )
            .into());
        }
        self.replace_state(&state)?;
        Ok(state)
    }

    pub fn export_material_markdown(
        &self,
        material_id: &str,
        destination: impl AsRef<Path>,
    ) -> Result<(), LibraryError> {
        let state = self.load()?;
        let material = state
            .materials
            .iter()
            .find(|item| item.id == material_id)
            .ok_or_else(|| DomainError::new("material_not_found", "Материал не найден"))?;
        let sources = material
            .idea_ids
            .iter()
            .filter_map(|id| state.ideas.iter().find(|idea| &idea.id == id))
            .map(|idea| {
                let book = state
                    .books
                    .iter()
                    .find(|book| book.id == idea.book_id)
                    .map(|book| book.title.as_str())
                    .unwrap_or("Книга");
                format!("- {book}, {} — {}", idea.section, idea.formulation)
            })
            .collect::<Vec<_>>()
            .join("\n");
        let markdown = format!("# {}\n\n## Проблема\n\n{}\n\n## Идея\n\n{}\n\n## Пример применения\n\n{}\n\n## Результат\n\n{}\n\n## Ограничения\n\n{}\n\n## Источники\n\n{}\n", material.title, material.problem, material.idea, material.example, material.result, material.limitations, sources);
        atomic_write(destination.as_ref(), markdown.as_bytes())?;
        Ok(())
    }

    pub fn export_draft_markdown(
        &self,
        draft_id: &str,
        destination: impl AsRef<Path>,
    ) -> Result<LibraryState, LibraryError> {
        let state = self.load()?;
        let draft = state
            .drafts
            .iter()
            .find(|item| item.id == draft_id)
            .ok_or_else(|| DomainError::new("draft_not_found", "Черновая заметка не найдена"))?;
        let book = state
            .books
            .iter()
            .find(|book| book.id == draft.book_id)
            .map(|book| book.title.as_str())
            .unwrap_or("Книга");
        let markdown = format!(
            "# Черновая заметка\n\nИсточник: {book}, {}, стр. {}\n\n> {}\n\n{}\n",
            draft.section, draft.page, draft.excerpt, draft.comment
        );
        atomic_write(destination.as_ref(), markdown.as_bytes())?;
        self.apply(LibraryAction::DiscardDraft {
            draft_id: draft_id.into(),
        })
    }

    fn create_snapshot(&self, state: &LibraryState) -> io::Result<()> {
        let dir = self.data_dir.join("snapshots");
        let path = dir.join(format!("snapshot-{}.json", unique_number()));
        atomic_write(&path, &serde_json::to_vec(state).map_err(io::Error::other)?)?;
        let mut snapshots: Vec<_> = fs::read_dir(&dir)?.filter_map(Result::ok).collect();
        snapshots.sort_by_key(|entry| entry.file_name());
        let remove_count = snapshots.len().saturating_sub(5);
        for entry in snapshots.into_iter().take(remove_count) {
            fs::remove_file(entry.path())?;
        }
        Ok(())
    }

    fn replace_state(&self, state: &LibraryState) -> io::Result<()> {
        let mut connection = Connection::open(&self.database_file).map_err(sqlite_io)?;
        let transaction = connection.transaction().map_err(sqlite_io)?;
        transaction.execute("INSERT INTO workspace_state (id, json) VALUES (1, ?1) ON CONFLICT(id) DO UPDATE SET json = excluded.json", [serde_json::to_string(state).map_err(io::Error::other)?]).map_err(sqlite_io)?;
        transaction
            .execute("DELETE FROM search_index", [])
            .map_err(sqlite_io)?;
        for book in &state.books {
            transaction.execute("INSERT INTO search_index (entity_id, kind, title, context) VALUES (?1, 'book', ?2, 'Название книги')", params![book.id, book.title]).map_err(sqlite_io)?;
        }
        for idea in &state.ideas {
            let book = state
                .books
                .iter()
                .find(|book| book.id == idea.book_id)
                .map(|book| book.title.as_str())
                .unwrap_or("Книга");
            let context = format!(
                "{book} · {} · {}",
                idea.section,
                idea.fragments
                    .iter()
                    .map(|item| item.excerpt.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            transaction.execute("INSERT INTO search_index (entity_id, kind, title, context) VALUES (?1, 'idea', ?2, ?3)", params![idea.id, idea.formulation, context]).map_err(sqlite_io)?;
        }
        transaction.commit().map_err(sqlite_io)
    }

    fn migrate(&self) -> io::Result<()> {
        let connection = Connection::open(&self.database_file).map_err(sqlite_io)?;
        connection.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; CREATE TABLE IF NOT EXISTS schema_migrations (version INTEGER PRIMARY KEY); CREATE TABLE IF NOT EXISTS workspace_state (id INTEGER PRIMARY KEY CHECK (id = 1), json TEXT NOT NULL); CREATE VIRTUAL TABLE IF NOT EXISTS search_index USING fts5(entity_id UNINDEXED, kind UNINDEXED, title, context, tokenize='unicode61'); INSERT OR IGNORE INTO schema_migrations(version) VALUES (1);").map_err(sqlite_io)?;
        let legacy = self.data_dir.join("library.json");
        let has_state: bool = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM workspace_state WHERE id = 1)",
                [],
                |row| row.get(0),
            )
            .map_err(sqlite_io)?;
        if !has_state && legacy.exists() {
            let state: LibraryState =
                serde_json::from_slice(&fs::read(&legacy)?).map_err(io::Error::other)?;
            drop(connection);
            self.replace_state(&state)?;
            fs::rename(&legacy, legacy.with_extension("json.migrated"))?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub context: String,
}

#[derive(Debug)]
pub enum LibraryError {
    Io(io::Error),
    Domain(DomainError),
}
impl From<io::Error> for LibraryError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
impl From<DomainError> for LibraryError {
    fn from(value: DomainError) -> Self {
        Self::Domain(value)
    }
}
impl std::fmt::Display for LibraryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => error.fmt(f),
            Self::Domain(error) => error.message.fmt(f),
        }
    }
}
impl std::error::Error for LibraryError {}

fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let temporary = path.with_extension("tmp");
    remove_if_present(&temporary)?;
    let mut file = fs::File::create(&temporary)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    fs::rename(temporary, path)
}
fn crypto_io(error: impl std::fmt::Display) -> io::Error {
    io::Error::other(error.to_string())
}
fn sqlite_io(error: rusqlite::Error) -> io::Error {
    io::Error::other(error)
}
fn cleanup_paths<'a>(paths: impl Iterator<Item = &'a PathBuf>) {
    for path in paths {
        let _ = remove_if_present(path);
    }
}
fn append_bytes<W: Write>(
    archive: &mut tar::Builder<W>,
    path: &str,
    bytes: &[u8],
) -> io::Result<()> {
    let mut header = tar::Header::new_gnu();
    header.set_path(path)?;
    header.set_size(bytes.len() as u64);
    header.set_mode(0o600);
    header.set_cksum();
    archive.append(&header, bytes)
}
fn remove_if_present(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_data_dir() -> PathBuf {
        std::env::temp_dir().join(format!("bookshelf-library-test-{}", unique_number()))
    }

    #[test]
    fn clean_launch_opens_an_empty_personal_library() {
        let data_dir = test_data_dir();
        let library = Library::open(&data_dir).unwrap();
        assert_eq!(library.load().unwrap(), LibraryState::default());
        fs::remove_dir_all(data_dir).unwrap();
    }

    #[test]
    fn workspace_change_survives_a_desktop_restart() {
        let data_dir = test_data_dir();
        Library::open(&data_dir)
            .unwrap()
            .save_workspace_note("Продолжить с главы 2".into())
            .unwrap();
        assert_eq!(
            Library::open(&data_dir)
                .unwrap()
                .load()
                .unwrap()
                .workspace_note,
            "Продолжить с главы 2"
        );
        fs::remove_dir_all(data_dir).unwrap();
    }

    #[test]
    fn repeated_workspace_changes_replace_state_portably() {
        let data_dir = test_data_dir();
        let library = Library::open(&data_dir).unwrap();
        library
            .save_workspace_note("Первая пометка".into())
            .unwrap();
        library
            .save_workspace_note("Актуальная пометка".into())
            .unwrap();
        assert_eq!(library.load().unwrap().workspace_note, "Актуальная пометка");
        assert!(data_dir.join("library.sqlite3").exists());
        fs::remove_dir_all(data_dir).unwrap();
    }

    #[test]
    fn failed_domain_action_does_not_replace_the_persisted_state() {
        let data_dir = test_data_dir();
        let library = Library::open(&data_dir).unwrap();
        library
            .save_workspace_note("Сохранённая пометка".into())
            .unwrap();
        let result = library.apply(LibraryAction::ResolveDraftAsIdea {
            draft_id: "missing".into(),
            formulation: "Формулировка".into(),
            section: "Глава".into(),
            assignments: vec!["recall".into()],
        });
        assert!(result.is_err());
        assert_eq!(
            library.load().unwrap().workspace_note,
            "Сохранённая пометка"
        );
        fs::remove_dir_all(data_dir).unwrap();
    }

    #[test]
    fn full_text_search_updates_in_the_same_transaction_as_working_state() {
        let data_dir = test_data_dir();
        let library = Library::open(&data_dir).unwrap();
        let mut state = LibraryState::default();
        state
            .books
            .push(Book::for_test("book", "Распределённые системы"));
        state.ideas.push(Idea {
            formulation: "Репликация требует явной модели согласованности".into(),
            ..Idea::for_test("idea", "book")
        });
        library.replace_state(&state).unwrap();
        assert_eq!(library.search("согласованности").unwrap()[0].id, "idea");
        state.ideas.clear();
        library.replace_state(&state).unwrap();
        assert!(library.search("согласованности").unwrap().is_empty());
        fs::remove_dir_all(data_dir).unwrap();
    }

    #[test]
    fn activating_another_study_preserves_work_and_keeps_only_one_active() {
        let mut state = LibraryState {
            books: vec![
                Book::for_test("one", "Первая"),
                Book::for_test("two", "Вторая"),
            ],
            ..LibraryState::default()
        };
        state
            .apply(LibraryAction::ActivateStudy {
                book_id: "one".into(),
            })
            .unwrap();
        state
            .apply(LibraryAction::ActivateStudy {
                book_id: "two".into(),
            })
            .unwrap();
        assert_eq!(state.active_study_book_id.as_deref(), Some("two"));
        assert_eq!(state.books.len(), 2);
    }

    #[test]
    fn resolving_a_draft_requires_an_authored_idea_source_and_assignment() {
        let mut state = LibraryState::default();
        state.books.push(Book::for_test("book", "Книга"));
        state.drafts.push(DraftNote::for_test("draft", "book"));
        let result = state.apply(LibraryAction::ResolveDraftAsIdea {
            draft_id: "draft".into(),
            formulation: "".into(),
            section: "Глава 1".into(),
            assignments: vec![],
        });
        assert_eq!(result.unwrap_err().code, "idea_fields_required");
        assert_eq!(state.drafts.len(), 1);
        assert!(state.ideas.is_empty());
    }

    #[test]
    fn a_negative_practical_experiment_is_a_valid_completed_experiment() {
        let mut state = LibraryState::default();
        state.ideas.push(Idea::for_test("idea", "book"));
        state
            .apply(LibraryAction::CompleteExperiment {
                idea_id: "idea".into(),
                situation: "Новый сервис".into(),
                action: "Применил паттерн".into(),
                result: "Усложнил поддержку".into(),
                conclusion: "Не применять для малого сервиса".into(),
                successful: false,
            })
            .unwrap();
        assert!(!state.experiments[0].successful);
        assert!(state.experiments[0].completed);
    }

    #[test]
    fn study_completion_requires_a_retrospective_and_three_to_seven_ideas() {
        let mut state = LibraryState::default();
        state.books.push(Book::for_test("book", "Книга"));
        state.active_study_book_id = Some("book".into());
        let result = state.apply(LibraryAction::CompleteStudy {
            book_id: "book".into(),
            retrospective: "".into(),
            significant_idea_ids: vec![],
            continuing_work: "".into(),
            debt_decision: "".into(),
        });
        assert_eq!(result.unwrap_err().code, "retrospective_required");
        assert_eq!(state.active_study_book_id.as_deref(), Some("book"));
    }

    #[test]
    fn encrypted_archive_round_trip_restores_state_and_rejects_a_wrong_password() {
        let source_dir = test_data_dir();
        let source = Library::open(&source_dir).unwrap();
        source
            .save_workspace_note("Переносимое состояние".into())
            .unwrap();
        let archive = source_dir.with_extension("bookshelf.age");
        source.export_archive(&archive, "надёжный пароль").unwrap();

        let target_dir = test_data_dir();
        let target = Library::open(&target_dir).unwrap();
        let wrong = target
            .import_archive(&archive, "другой пароль")
            .unwrap_err();
        assert!(
            matches!(wrong, LibraryError::Domain(ref error) if error.code == "archive_password_invalid")
        );
        assert_eq!(target.load().unwrap(), LibraryState::default());

        target.import_archive(&archive, "надёжный пароль").unwrap();
        assert_eq!(
            target.load().unwrap().workspace_note,
            "Переносимое состояние"
        );
        fs::remove_dir_all(source_dir).unwrap();
        fs::remove_dir_all(target_dir).unwrap();
        fs::remove_file(archive).unwrap();
    }
}
