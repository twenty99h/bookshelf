use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
use ts_rs::TS;

mod drafts;
mod knowledge;
mod practice;
mod reading;
mod rules;
mod study;

pub use drafts::{DraftNote, SourceFragment};
pub use knowledge::{
    Idea, IdeaAssignment, IdeaLink, IdeaRelation, IdeaReview, IdeaVersion, ReviewDecision,
    ReviewKind, Topic, TransferMaterial,
};
pub use practice::{Experiment, ExperimentDraft, ExperimentStatus, Recall, RecallRating};
#[allow(unused_imports)]
pub use reading::{
    Book, DocumentMode, OutlineItem, ReaderPreferences, ReaderSidebarTab, ReadingPosition,
};
#[allow(unused_imports)]
pub use study::{
    CompletionWorkDecision, CompletionWorkKind, MilestoneKind, Retrospective, StudyCompletionDraft,
    StudyCycle, StudyMilestone, StudyStatus,
};

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
    pub experiment_drafts: Vec<ExperimentDraft>,
    pub recalls: Vec<Recall>,
    pub materials: Vec<TransferMaterial>,
    pub reviews: Vec<IdeaReview>,
    pub milestones: Vec<StudyMilestone>,
    pub completion_drafts: Vec<StudyCompletionDraft>,
    pub workspace_note: String,
    pub active_study_book_id: Option<String>,
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
    CaptureDraftSources {
        book_id: String,
        section: String,
        fragments: Vec<SourceFragment>,
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
    DeferDraft {
        draft_id: String,
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
    ArchiveBook {
        book_id: String,
    },
    RestoreBook {
        book_id: String,
    },
    DeleteBook {
        book_id: String,
    },
    StartRepeatStudy {
        book_id: String,
    },
    UpdateReaderPreferences {
        book_id: String,
        preferences: ReaderPreferences,
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
    CreateExperiment {
        idea_id: String,
        situation: String,
        action: String,
        next_step: String,
    },
    SaveExperimentDraft {
        draft: ExperimentDraft,
    },
    CompleteExperiment {
        idea_id: String,
        situation: String,
        action: String,
        result: String,
        conclusion: String,
    },
    AdvanceExperiment {
        experiment_id: String,
        status: ExperimentStatus,
        situation: String,
        action: String,
        result: String,
        conclusion: String,
        cancellation_reason: String,
        next_step: String,
    },
    CompleteRecall {
        recall_id: String,
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
        unfinished_work_decision: String,
        work_decisions: Vec<CompletionWorkDecision>,
    },
    SaveStudyCompletionDraft {
        draft: StudyCompletionDraft,
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
