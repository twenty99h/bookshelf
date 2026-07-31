use super::*;

pub(super) fn resolution_milestones(
    book_id: String,
    timestamp: u64,
    make_id: &mut impl FnMut(&str) -> String,
) -> [StudyMilestone; 2] {
    [
        StudyMilestone {
            id: make_id("milestone"),
            book_id: book_id.clone(),
            kind: MilestoneKind::DraftResolved,
            occurred_at: timestamp,
            page: None,
        },
        StudyMilestone {
            id: make_id("milestone"),
            book_id,
            kind: MilestoneKind::IdeaFormulated,
            occurred_at: timestamp,
            page: None,
        },
    ]
}
