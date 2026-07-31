use super::ExperimentStatus;

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
