use super::*;

impl LibraryState {
    #[cfg(test)]
    pub fn apply(&mut self, action: LibraryAction) -> Result<(), DomainError> {
        let timestamp = now();
        self.apply_with(action, timestamp, &mut |prefix| new_id(prefix))
    }

    pub fn apply_with(
        &mut self,
        action: LibraryAction,
        timestamp: u64,
        make_id: &mut impl FnMut(&str) -> String,
    ) -> Result<(), DomainError> {
        match &action {
            LibraryAction::UpdateReading { .. }
            | LibraryAction::SaveOutline { .. }
            | LibraryAction::UpdateReaderPreferences { .. } => {
                reading::apply(self, action, timestamp, make_id)
            }
            LibraryAction::CaptureDraft { .. }
            | LibraryAction::CaptureDraftSources { .. }
            | LibraryAction::ResolveDraftAsIdea { .. }
            | LibraryAction::AttachDraftToIdea { .. }
            | LibraryAction::DeferDraft { .. }
            | LibraryAction::DiscardDraft { .. } => drafts::apply(self, action, timestamp, make_id),
            LibraryAction::UpdateIdea { .. }
            | LibraryAction::CreateTopic { .. }
            | LibraryAction::AssignTopic { .. }
            | LibraryAction::ConfirmSuggestedTopic { .. }
            | LibraryAction::LinkIdeas { .. }
            | LibraryAction::SaveMaterial { .. }
            | LibraryAction::RecordReviewResponse { .. }
            | LibraryAction::ResolveReview { .. } => {
                knowledge::apply(self, action, timestamp, make_id)
            }
            LibraryAction::CreateExperiment { .. }
            | LibraryAction::SaveExperimentDraft { .. }
            | LibraryAction::CompleteExperiment { .. }
            | LibraryAction::AdvanceExperiment { .. }
            | LibraryAction::CompleteRecall { .. }
            | LibraryAction::RescheduleRecall { .. } => {
                practice::apply(self, action, timestamp, make_id)
            }
            LibraryAction::SaveWorkspaceNote { .. }
            | LibraryAction::ActivateStudy { .. }
            | LibraryAction::CompleteReading { .. }
            | LibraryAction::ArchiveBook { .. }
            | LibraryAction::RestoreBook { .. }
            | LibraryAction::DeleteBook { .. }
            | LibraryAction::StartRepeatStudy { .. }
            | LibraryAction::CompleteStudy { .. }
            | LibraryAction::SaveStudyCompletionDraft { .. } => {
                study::apply(self, action, timestamp, make_id)
            }
        }
    }
}
