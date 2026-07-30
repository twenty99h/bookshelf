use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
use ts_rs::TS;

mod rules;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(export)]
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
    pub debt_reminder_days: u16,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
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
    pub(crate) fn for_test(id: &str, title: &str) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            has_text_layer: true,
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct OutlineItem {
    pub id: String,
    pub title: String,
    pub page: u32,
    pub parent_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
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

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
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
    pub(crate) fn for_test(id: &str, book_id: &str) -> Self {
        Self {
            id: id.into(),
            book_id: book_id.into(),
            section: "Глава 1".into(),
            excerpt: "Фрагмент".into(),
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Idea {
    pub id: String,
    pub book_id: String,
    pub section: String,
    pub formulation: String,
    pub assignments: Vec<IdeaAssignment>,
    pub fragments: Vec<SourceFragment>,
    pub versions: Vec<IdeaVersion>,
    pub topic_ids: Vec<String>,
}

impl Idea {
    #[cfg(test)]
    pub(crate) fn for_test(id: &str, book_id: &str) -> Self {
        Self {
            id: id.into(),
            book_id: book_id.into(),
            section: "Глава 1".into(),
            formulation: "Идея".into(),
            assignments: vec![IdeaAssignment::Recall],
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SourceFragment {
    pub page: u32,
    pub excerpt: String,
    pub context: String,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct IdeaVersion {
    pub formulation: String,
    pub saved_at: u64,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Topic {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct IdeaLink {
    pub id: String,
    pub from_idea_id: String,
    pub to_idea_id: String,
    pub relation: IdeaRelation,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
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

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Recall {
    pub id: String,
    pub idea_id: String,
    pub answer: String,
    pub rating: RecallRating,
    pub next_at: u64,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct StudySession {
    pub id: String,
    pub intention: String,
    pub planned_at: u64,
    pub status: SessionStatus,
    pub resolution_reason: String,
    pub debt_at_start: usize,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
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

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct IdeaReview {
    pub id: String,
    pub idea_id: String,
    pub request_kind: ReviewKind,
    pub response: String,
    pub decision: ReviewDecision,
    pub conclusion: String,
    pub pending: bool,
    pub reviewed_at: u64,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum ReviewKind {
    #[default]
    IdeaReview,
    RecallGaps,
    TopicSuggestion,
    LinkSuggestion,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum ReviewDecision {
    #[default]
    Pending,
    Refined,
    Unchanged,
    Later,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum IdeaAssignment {
    Recall,
    Transfer,
    Experiment,
    Mastered,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum IdeaRelation {
    #[default]
    Complements,
    Clarifies,
    Contradicts,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum RecallRating {
    #[default]
    Confident,
    Partial,
    NotRecalled,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum SessionStatus {
    #[default]
    Planned,
    Active,
    Completed,
    Moved,
    Replaced,
    Cancelled,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Retrospective {
    pub text: String,
    pub significant_idea_ids: Vec<String>,
    pub continuing_work: String,
    pub debt_decision: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[ts(export)]
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
        assignments: Vec<IdeaAssignment>,
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
    SetDebtReminder {
        days: u16,
    },
    StartSession {
        session_id: String,
    },
    PlanSession {
        intention: String,
        planned_at: u64,
    },
    ResolveSession {
        session_id: String,
        status: SessionStatus,
        reason: String,
    },
    UpdateIdea {
        idea_id: String,
        formulation: String,
        assignments: Vec<IdeaAssignment>,
    },
    CreateTopic {
        name: String,
    },
    AssignTopic {
        idea_id: String,
        topic_id: String,
    },
    ConfirmSuggestedTopic {
        idea_id: String,
        name: String,
    },
    LinkIdeas {
        from_idea_id: String,
        to_idea_id: String,
        relation: IdeaRelation,
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
        rating: RecallRating,
        #[ts(optional = nullable)]
        next_at: Option<u64>,
    },
    RescheduleRecall {
        recall_id: String,
        next_at: u64,
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
    RecordReviewResponse {
        idea_id: String,
        request_kind: ReviewKind,
        response: String,
    },
    ResolveReview {
        idea_id: String,
        request_kind: ReviewKind,
        decision: ReviewDecision,
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum DomainErrorKind {
    Validation,
    NotFound,
    Conflict,
    DataIntegrity,
}

#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum DomainError {
    #[error("{message}")]
    Validation { code: &'static str, message: String },
    #[error("{message}")]
    NotFound { code: &'static str, message: String },
    #[error("{message}")]
    Conflict { code: &'static str, message: String },
    #[error("{message}")]
    DataIntegrity { code: &'static str, message: String },
}

impl DomainError {
    pub(crate) fn new(code: &'static str, message: &str) -> Self {
        let fields = || (code, message.into());
        if code.ends_with("_not_found") {
            let (code, message) = fields();
            Self::NotFound { code, message }
        } else if code.ends_with("_corrupt")
            || code.ends_with("_inconsistent")
            || code.ends_with("_unsupported")
        {
            let (code, message) = fields();
            Self::DataIntegrity { code, message }
        } else if code.ends_with("_changed")
            || code.ends_with("_duplicates")
            || code.ends_with("_not_empty")
        {
            let (code, message) = fields();
            Self::Conflict { code, message }
        } else {
            let (code, message) = fields();
            Self::Validation { code, message }
        }
    }

    #[cfg(test)]
    pub(crate) fn code(&self) -> &'static str {
        match self {
            Self::Validation { code, .. }
            | Self::NotFound { code, .. }
            | Self::Conflict { code, .. }
            | Self::DataIntegrity { code, .. } => code,
        }
    }

    pub(crate) fn kind(&self) -> DomainErrorKind {
        match self {
            Self::Validation { .. } => DomainErrorKind::Validation,
            Self::NotFound { .. } => DomainErrorKind::NotFound,
            Self::Conflict { .. } => DomainErrorKind::Conflict,
            Self::DataIntegrity { .. } => DomainErrorKind::DataIntegrity,
        }
    }

    pub(crate) fn into_message(self) -> String {
        match self {
            Self::Validation { message, .. }
            | Self::NotFound { message, .. }
            | Self::Conflict { message, .. }
            | Self::DataIntegrity { message, .. } => message,
        }
    }
}

pub(crate) fn find_book<'a>(state: &'a LibraryState, id: &str) -> Result<&'a Book, DomainError> {
    state
        .books
        .iter()
        .find(|item| item.id == id)
        .ok_or_else(|| DomainError::new("book_not_found", "Книга не найдена"))
}
pub(crate) fn find_book_mut<'a>(
    state: &'a mut LibraryState,
    id: &str,
) -> Result<&'a mut Book, DomainError> {
    state
        .books
        .iter_mut()
        .find(|item| item.id == id)
        .ok_or_else(|| DomainError::new("book_not_found", "Книга не найдена"))
}

pub(crate) fn book_title<'a>(state: &'a LibraryState, book_id: &str) -> &'a str {
    state
        .books
        .iter()
        .find(|book| book.id == book_id)
        .map(|book| book.title.as_str())
        .unwrap_or("Книга")
}
pub(crate) fn find_idea<'a>(state: &'a LibraryState, id: &str) -> Result<&'a Idea, DomainError> {
    state
        .ideas
        .iter()
        .find(|item| item.id == id)
        .ok_or_else(|| DomainError::new("idea_not_found", "Идея не найдена"))
}
pub(crate) fn find_idea_mut<'a>(
    state: &'a mut LibraryState,
    id: &str,
) -> Result<&'a mut Idea, DomainError> {
    state
        .ideas
        .iter_mut()
        .find(|item| item.id == id)
        .ok_or_else(|| DomainError::new("idea_not_found", "Идея не найдена"))
}
pub(crate) fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
pub(crate) fn unique_number() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        + NEXT_ID.fetch_add(1, Ordering::Relaxed) as u128
}
pub(crate) fn new_id(prefix: &str) -> String {
    format!("{prefix}-{}", unique_number())
}
