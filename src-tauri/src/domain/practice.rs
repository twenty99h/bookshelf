use super::ExperimentStatus;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

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
