use super::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SourceFragment {
    pub page: u32,
    pub excerpt: String,
    pub context: String,
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
    pub fragments: Vec<SourceFragment>,
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

pub(super) fn apply(
    state: &mut LibraryState,
    action: LibraryAction,
    timestamp: u64,
    make_id: &mut impl FnMut(&str) -> String,
) -> Result<(), DomainError> {
    match action {
        LibraryAction::CaptureDraft {
            book_id,
            section,
            page,
            excerpt,
            context,
            comment,
        } => {
            ensure_book_allows_capture(state, &book_id)?;
            if excerpt.trim().is_empty() || page == 0 {
                return Err(DomainError::new(
                    "draft_source_required",
                    "Выберите фрагмент и страницу",
                ));
            }
            state.drafts.push(DraftNote {
                id: make_id("draft"),
                book_id: book_id.clone(),
                section,
                page,
                excerpt: excerpt.clone(),
                context: context.clone(),
                comment,
                fragments: vec![SourceFragment {
                    page,
                    excerpt: excerpt.clone(),
                    context: context.clone(),
                }],
                created_at: timestamp,
            });
            state.milestones.push(StudyMilestone {
                id: make_id("milestone"),
                book_id,
                kind: MilestoneKind::DraftCaptured,
                occurred_at: timestamp,
                page: Some(page),
            });
        }
        LibraryAction::CaptureDraftSources {
            book_id,
            section,
            fragments,
            comment,
        } => {
            ensure_book_allows_capture(state, &book_id)?;
            if fragments.is_empty()
                || fragments
                    .iter()
                    .any(|fragment| fragment.page == 0 || fragment.excerpt.trim().is_empty())
            {
                return Err(DomainError::new(
                    "draft_source_required",
                    "Добавьте хотя бы один адресуемый источник",
                ));
            }
            let first = fragments[0].clone();
            state.drafts.push(DraftNote {
                id: make_id("draft"),
                book_id: book_id.clone(),
                section,
                page: first.page,
                excerpt: first.excerpt,
                context: first.context,
                comment,
                fragments,
                created_at: timestamp,
            });
            state.milestones.push(StudyMilestone {
                id: make_id("milestone"),
                book_id,
                kind: MilestoneKind::DraftCaptured,
                occurred_at: timestamp,
                page: Some(first.page),
            });
        }
        LibraryAction::ResolveDraftAsIdea {
            draft_id,
            formulation,
            section,
            assignments,
        } => {
            if formulation.trim().is_empty() || section.trim().is_empty() {
                return Err(DomainError::new(
                    "idea_fields_required",
                    "Нужны авторская формулировка и раздел",
                ));
            }
            let draft = state
                .drafts
                .iter()
                .find(|item| item.id == draft_id)
                .cloned()
                .ok_or_else(|| {
                    DomainError::new("draft_not_found", "Черновая заметка не найдена")
                })?;
            let book_id = draft.book_id.clone();
            let idea = Idea {
                id: make_id("idea"),
                book_id: book_id.clone(),
                section,
                formulation: formulation.clone(),
                assignments,
                fragments: if draft.fragments.is_empty() {
                    vec![SourceFragment {
                        page: draft.page,
                        excerpt: draft.excerpt,
                        context: draft.context,
                    }]
                } else {
                    draft.fragments
                },
                versions: vec![IdeaVersion {
                    formulation,
                    saved_at: timestamp,
                }],
                topic_ids: vec![],
            };
            state.ideas.push(idea);
            state.drafts.retain(|item| item.id != draft_id);
            state
                .milestones
                .extend(resolution_milestones(book_id, timestamp, make_id));
        }
        LibraryAction::AttachDraftToIdea { draft_id, idea_id } => {
            let draft = state
                .drafts
                .iter()
                .find(|item| item.id == draft_id)
                .cloned()
                .ok_or_else(|| {
                    DomainError::new("draft_not_found", "Черновая заметка не найдена")
                })?;
            let idea = find_idea_mut(state, &idea_id)?;
            if draft.fragments.is_empty() {
                idea.fragments.push(SourceFragment {
                    page: draft.page,
                    excerpt: draft.excerpt,
                    context: draft.context,
                });
            } else {
                idea.fragments.extend(draft.fragments);
            }
            state.drafts.retain(|item| item.id != draft_id);
        }
        LibraryAction::DeferDraft { draft_id } => {
            let index = state
                .drafts
                .iter()
                .position(|item| item.id == draft_id)
                .ok_or_else(|| {
                    DomainError::new("draft_not_found", "Черновая заметка не найдена")
                })?;
            let draft = state.drafts.remove(index);
            state.drafts.push(draft);
        }
        LibraryAction::DiscardDraft { draft_id } => {
            if !state.drafts.iter().any(|item| item.id == draft_id) {
                return Err(DomainError::new(
                    "draft_not_found",
                    "Черновая заметка не найдена",
                ));
            }
            state.drafts.retain(|item| item.id != draft_id);
        }
        _ => unreachable!("action dispatched to the wrong capability"),
    }
    Ok(())
}
fn ensure_book_allows_capture(state: &LibraryState, book_id: &str) -> Result<(), DomainError> {
    let book = find_book(state, book_id)?;
    if book.study_status == StudyStatus::Completed {
        return Err(DomainError::new(
            "study_reactivation_required",
            "Начните повторное изучение, чтобы создавать новые черновые заметки",
        ));
    }
    Ok(())
}
