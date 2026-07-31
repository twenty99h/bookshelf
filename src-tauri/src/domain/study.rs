use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use ts_rs::TS;

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum StudyStatus {
    #[default]
    Ready,
    Active,
    Paused,
    ReadyToComplete,
    Completed,
    Repeating,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct StudyCycle {
    pub id: String,
    pub started_at: u64,
    pub completed_at: Option<u64>,
    pub retrospective: Option<Retrospective>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct Retrospective {
    pub text: String,
    pub significant_idea_ids: Vec<String>,
    pub continuing_work: String,
    pub unfinished_work_decision: String,
    pub work_decisions: Vec<CompletionWorkDecision>,
}
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct StudyCompletionDraft {
    pub book_id: String,
    pub step: u8,
    pub reading_confirmed: bool,
    pub significant_idea_ids: Vec<String>,
    pub retrospective: String,
    pub unfinished_work_decision: String,
    pub continuing_work: String,
    pub work_decisions: Vec<CompletionWorkDecision>,
}
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CompletionWorkDecision {
    pub work_id: String,
    pub kind: CompletionWorkKind,
    pub decision: String,
}
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum CompletionWorkKind {
    #[default]
    Draft,
    Review,
    Recall,
    Experiment,
}
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct StudyMilestone {
    pub id: String,
    pub book_id: String,
    pub kind: MilestoneKind,
    pub occurred_at: u64,
    pub page: Option<u32>,
}
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum MilestoneKind {
    ReadingProgress,
    DraftCaptured,
    DraftResolved,
    IdeaFormulated,
    RecallCompleted,
    ExperimentAdvanced,
    StudyCompleted,
}

pub(super) fn apply(
    state: &mut LibraryState,
    action: LibraryAction,
    timestamp: u64,
    make_id: &mut impl FnMut(&str) -> String,
) -> Result<(), DomainError> {
    match action {
        LibraryAction::SaveWorkspaceNote { note } => {
            if note.chars().count() > 240 {
                return Err(DomainError::new(
                    "workspace_note_too_long",
                    "Пометка не может быть длиннее 240 символов",
                ));
            }
            state.workspace_note = note;
        }
        LibraryAction::ActivateStudy { book_id } => {
            find_book(state, &book_id)?;
            if let Some(active_id) = state.active_study_book_id.clone() {
                if active_id != book_id {
                    find_book_mut(state, &active_id)?.study_status = StudyStatus::Paused;
                }
            }
            let book = find_book_mut(state, &book_id)?;
            book.study_status = if book.study_cycles.is_empty() {
                StudyStatus::Active
            } else {
                StudyStatus::Repeating
            };
            if book.study_cycles.is_empty() {
                book.study_cycles.push(StudyCycle {
                    id: make_id("study-cycle"),
                    started_at: timestamp,
                    completed_at: None,
                    retrospective: None,
                });
            }
            state.active_study_book_id = Some(book_id);
        }
        LibraryAction::CompleteReading { book_id } => {
            let book = find_book_mut(state, &book_id)?;
            book.reading_completed = true;
            book.study_status = StudyStatus::ReadyToComplete;
        }
        LibraryAction::ArchiveBook { book_id } => {
            let was_active = state.active_study_book_id.as_deref() == Some(book_id.as_str());
            let book = find_book_mut(state, &book_id)?;
            book.archived = true;
            if was_active {
                book.study_status = StudyStatus::Paused;
                state.active_study_book_id = None;
            }
        }
        LibraryAction::RestoreBook { book_id } => {
            let book = find_book_mut(state, &book_id)?;
            book.archived = false;
        }
        LibraryAction::DeleteBook { book_id } => {
            find_book(state, &book_id)?;
            let idea_ids = state
                .ideas
                .iter()
                .filter(|idea| idea.book_id == book_id)
                .map(|idea| idea.id.clone())
                .collect::<HashSet<_>>();

            state.books.retain(|book| book.id != book_id);
            state.drafts.retain(|draft| draft.book_id != book_id);
            state.ideas.retain(|idea| idea.book_id != book_id);
            state.idea_links.retain(|link| {
                !idea_ids.contains(&link.from_idea_id) && !idea_ids.contains(&link.to_idea_id)
            });
            state
                .experiments
                .retain(|experiment| !idea_ids.contains(&experiment.idea_id));
            state
                .recalls
                .retain(|recall| !idea_ids.contains(&recall.idea_id));
            state
                .reviews
                .retain(|review| !idea_ids.contains(&review.idea_id));
            for material in &mut state.materials {
                material
                    .idea_ids
                    .retain(|idea_id| !idea_ids.contains(idea_id));
            }
            state
                .materials
                .retain(|material| !material.idea_ids.is_empty());
            state
                .milestones
                .retain(|milestone| milestone.book_id != book_id);
            state
                .completion_drafts
                .retain(|draft| draft.book_id != book_id);
            if state.active_study_book_id.as_deref() == Some(book_id.as_str()) {
                state.active_study_book_id = None;
            }
        }
        LibraryAction::StartRepeatStudy { book_id } => {
            if find_book(state, &book_id)?.study_status != StudyStatus::Completed {
                return Err(DomainError::new(
                    "repeat_study_requires_completion",
                    "Повторное изучение доступно после завершённого цикла",
                ));
            }
            if let Some(active_id) = state.active_study_book_id.clone() {
                if active_id != book_id {
                    find_book_mut(state, &active_id)?.study_status = StudyStatus::Paused;
                }
            }
            let book = find_book_mut(state, &book_id)?;
            book.reading_completed = false;
            book.study_status = StudyStatus::Repeating;
            book.study_cycles.push(StudyCycle {
                id: make_id("study-cycle"),
                started_at: timestamp,
                completed_at: None,
                retrospective: None,
            });
            state.active_study_book_id = Some(book_id);
        }
        LibraryAction::CompleteStudy {
            book_id,
            retrospective,
            significant_idea_ids,
            continuing_work,
            unfinished_work_decision,
            work_decisions,
        } => {
            find_book(state, &book_id)?;
            if retrospective.trim().is_empty()
                || !(3..=7).contains(&significant_idea_ids.len())
                || unfinished_work_decision.trim().is_empty()
            {
                return Err(DomainError::new(
                    "retrospective_required",
                    "Добавьте ретроспективу, 3–7 значимых идей и решение по незавершённой работе",
                ));
            }
            if significant_idea_ids.iter().any(|id| {
                !state
                    .ideas
                    .iter()
                    .any(|idea| &idea.id == id && idea.book_id == book_id)
            }) {
                return Err(DomainError::new(
                    "retrospective_ideas_invalid",
                    "Значимые идеи должны относиться к этой книге",
                ));
            }
            let idea_belongs_to_book = |idea_id: &str| {
                state
                    .ideas
                    .iter()
                    .any(|idea| idea.id == idea_id && idea.book_id == book_id)
            };
            let mut required_work = state
                .drafts
                .iter()
                .filter(|draft| draft.book_id == book_id)
                .map(|draft| (draft.id.as_str(), CompletionWorkKind::Draft))
                .collect::<Vec<_>>();
            required_work.extend(
                state
                    .reviews
                    .iter()
                    .filter(|review| review.pending && idea_belongs_to_book(&review.idea_id))
                    .map(|review| (review.id.as_str(), CompletionWorkKind::Review)),
            );
            required_work.extend(
                state
                    .recalls
                    .iter()
                    .filter(|recall| idea_belongs_to_book(&recall.idea_id))
                    .map(|recall| (recall.id.as_str(), CompletionWorkKind::Recall)),
            );
            required_work.extend(
                state
                    .experiments
                    .iter()
                    .filter(|experiment| {
                        !matches!(
                            experiment.status,
                            ExperimentStatus::Completed | ExperimentStatus::Cancelled
                        ) && idea_belongs_to_book(&experiment.idea_id)
                    })
                    .map(|experiment| (experiment.id.as_str(), CompletionWorkKind::Experiment)),
            );
            let decisions_complete = required_work.iter().all(|(work_id, kind)| {
                work_decisions
                    .iter()
                    .filter(|decision| decision.work_id == *work_id && decision.kind == *kind)
                    .count()
                    == 1
                    && work_decisions.iter().any(|decision| {
                        decision.work_id == *work_id
                            && decision.kind == *kind
                            && !decision.decision.trim().is_empty()
                    })
            });
            if !decisions_complete {
                return Err(DomainError::new(
                        "completion_work_decisions_required",
                        "Выберите отдельное решение для каждого черновика, проверки, восстановления и эксперимента",
                    ));
            }
            let book = find_book_mut(state, &book_id)?;
            book.study_status = StudyStatus::Completed;
            let completed_retrospective = Retrospective {
                text: retrospective,
                significant_idea_ids,
                continuing_work,
                unfinished_work_decision,
                work_decisions,
            };
            book.retrospective = Some(completed_retrospective.clone());
            if let Some(cycle) = book.study_cycles.last_mut() {
                cycle.completed_at = Some(timestamp);
                cycle.retrospective = Some(completed_retrospective);
            }
            if state.active_study_book_id.as_deref() == Some(book_id.as_str()) {
                state.active_study_book_id = None;
            }
            state
                .completion_drafts
                .retain(|draft| draft.book_id != book_id);
            state.milestones.push(StudyMilestone {
                id: make_id("milestone"),
                book_id,
                kind: MilestoneKind::StudyCompleted,
                occurred_at: timestamp,
                page: None,
            });
        }
        LibraryAction::SaveStudyCompletionDraft { draft } => {
            find_book(state, &draft.book_id)?;
            if !(1..=6).contains(&draft.step) {
                return Err(DomainError::new(
                    "completion_step_invalid",
                    "Шаг завершения должен быть от 1 до 6",
                ));
            }
            state
                .completion_drafts
                .retain(|existing| existing.book_id != draft.book_id);
            state.completion_drafts.push(draft);
        }
        _ => unreachable!("action dispatched to the wrong capability"),
    }
    Ok(())
}
