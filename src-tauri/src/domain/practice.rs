use super::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Experiment {
    pub id: String,
    pub idea_id: String,
    pub situation: String,
    pub action: String,
    pub result: String,
    pub conclusion: String,
    pub status: ExperimentStatus,
    pub cancellation_reason: String,
    pub next_step: String,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum ExperimentStatus {
    #[default]
    Intent,
    Running,
    Reviewing,
    Completed,
    Cancelled,
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

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum RecallRating {
    #[default]
    Confident,
    Partial,
    NotRecalled,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct ExperimentDraft {
    pub id: String,
    pub idea_id: String,
    pub situation: String,
    pub action: String,
    pub next_step: String,
}

pub(super) fn valid_experiment_transition(from: ExperimentStatus, to: ExperimentStatus) -> bool {
    if from == to {
        return !matches!(
            from,
            ExperimentStatus::Completed | ExperimentStatus::Cancelled
        );
    }
    matches!(
        (from, to),
        (ExperimentStatus::Intent, ExperimentStatus::Running)
            | (ExperimentStatus::Intent, ExperimentStatus::Cancelled)
            | (ExperimentStatus::Running, ExperimentStatus::Reviewing)
            | (ExperimentStatus::Running, ExperimentStatus::Cancelled)
            | (ExperimentStatus::Reviewing, ExperimentStatus::Running)
            | (ExperimentStatus::Reviewing, ExperimentStatus::Completed)
            | (ExperimentStatus::Reviewing, ExperimentStatus::Cancelled)
    )
}

pub(super) fn apply(
    state: &mut LibraryState,
    action: LibraryAction,
    timestamp: u64,
    make_id: &mut impl FnMut(&str) -> String,
) -> Result<(), DomainError> {
    match action {
        LibraryAction::CreateExperiment {
            idea_id,
            situation,
            action,
            next_step,
        } => {
            let book_id = find_idea(state, &idea_id)?.book_id.clone();
            if situation.trim().is_empty() || action.trim().is_empty() {
                return Err(DomainError::new(
                    "experiment_intent_required",
                    "Опишите ситуацию и проверяемое действие",
                ));
            }
            state.experiments.push(Experiment {
                id: make_id("experiment"),
                idea_id: idea_id.clone(),
                situation,
                action,
                result: String::new(),
                conclusion: String::new(),
                status: ExperimentStatus::Intent,
                cancellation_reason: String::new(),
                next_step,
            });
            state.milestones.push(StudyMilestone {
                id: make_id("milestone"),
                book_id,
                kind: MilestoneKind::ExperimentAdvanced,
                occurred_at: timestamp,
                page: None,
            });
            state
                .experiment_drafts
                .retain(|draft| draft.idea_id != idea_id);
        }
        LibraryAction::SaveExperimentDraft { draft } => {
            find_idea(state, &draft.idea_id)?;
            if let Some(saved) = state
                .experiment_drafts
                .iter_mut()
                .find(|saved| saved.id == draft.id)
            {
                *saved = draft;
            } else {
                state.experiment_drafts.push(draft);
            }
        }
        LibraryAction::CompleteExperiment {
            idea_id,
            situation,
            action,
            result,
            conclusion,
        } => {
            let book_id = find_idea(state, &idea_id)?.book_id.clone();
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
            state.experiments.push(Experiment {
                id: make_id("experiment"),
                idea_id,
                situation,
                action,
                result,
                conclusion,
                status: ExperimentStatus::Completed,
                cancellation_reason: String::new(),
                next_step: String::new(),
            });
            state.milestones.push(StudyMilestone {
                id: make_id("milestone"),
                book_id,
                kind: MilestoneKind::ExperimentAdvanced,
                occurred_at: timestamp,
                page: None,
            });
        }
        LibraryAction::AdvanceExperiment {
            experiment_id,
            status,
            situation,
            action,
            result,
            conclusion,
            cancellation_reason,
            next_step,
        } => {
            let current_status = state
                .experiments
                .iter()
                .find(|item| item.id == experiment_id)
                .map(|item| item.status)
                .ok_or_else(|| DomainError::new("experiment_not_found", "Эксперимент не найден"))?;
            if !valid_experiment_transition(current_status, status) {
                return Err(DomainError::new(
                    "experiment_transition_invalid",
                    "Выберите следующий допустимый этап эксперимента",
                ));
            }
            if status == ExperimentStatus::Cancelled && cancellation_reason.trim().is_empty() {
                return Err(DomainError::new(
                    "experiment_cancellation_reason_required",
                    "Укажите причину отмены замысла",
                ));
            }
            if status == ExperimentStatus::Completed
                && [
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
                    "Для завершения нужны ситуация, действие, наблюдаемый результат и вывод",
                ));
            }
            let status_changed = current_status != status;
            let idea_id = {
                let experiment = state
                    .experiments
                    .iter_mut()
                    .find(|item| item.id == experiment_id)
                    .ok_or_else(|| {
                        DomainError::new("experiment_not_found", "Эксперимент не найден")
                    })?;
                experiment.status = status;
                experiment.situation = situation;
                experiment.action = action;
                experiment.result = result;
                experiment.conclusion = conclusion;
                experiment.cancellation_reason = cancellation_reason;
                experiment.next_step = next_step;
                experiment.idea_id.clone()
            };
            if status_changed {
                let book_id = find_idea(state, &idea_id)?.book_id.clone();
                state.milestones.push(StudyMilestone {
                    id: make_id("milestone"),
                    book_id,
                    kind: MilestoneKind::ExperimentAdvanced,
                    occurred_at: timestamp,
                    page: None,
                });
            }
        }
        LibraryAction::CompleteRecall {
            recall_id,
            answer,
            rating,
            next_at,
        } => {
            if answer.trim().is_empty() {
                return Err(DomainError::new(
                    "recall_invalid",
                    "Напишите ответ и выберите самооценку",
                ));
            }
            let suggested_days = match rating {
                RecallRating::Confident => 30,
                RecallRating::Partial => 7,
                RecallRating::NotRecalled => 1,
            };
            let recall = state
                .recalls
                .iter_mut()
                .find(|item| item.id == recall_id)
                .ok_or_else(|| DomainError::new("recall_not_found", "Восстановление не найдено"))?;
            let idea_id = recall.idea_id.clone();
            recall.answer = answer;
            recall.rating = rating;
            recall.next_at = next_at.unwrap_or_else(|| timestamp + suggested_days * 86_400);
            let book_id = find_idea(state, &idea_id)?.book_id.clone();
            state.milestones.push(StudyMilestone {
                id: make_id("milestone"),
                book_id,
                kind: MilestoneKind::RecallCompleted,
                occurred_at: timestamp,
                page: None,
            });
        }
        LibraryAction::RescheduleRecall { recall_id, next_at } => {
            if next_at == 0 {
                return Err(DomainError::new(
                    "recall_schedule_invalid",
                    "Выберите дату следующего восстановления",
                ));
            }
            let recall = state
                .recalls
                .iter_mut()
                .find(|item| item.id == recall_id)
                .ok_or_else(|| DomainError::new("recall_not_found", "Восстановление не найдено"))?;
            recall.next_at = next_at;
        }
        _ => unreachable!("action dispatched to the wrong capability"),
    }
    Ok(())
}
