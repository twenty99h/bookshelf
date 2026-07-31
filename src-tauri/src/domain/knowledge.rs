use super::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

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

pub(super) fn validate_link(
    state: &LibraryState,
    from_idea_id: &str,
    to_idea_id: &str,
) -> Result<(), DomainError> {
    if from_idea_id == to_idea_id {
        return Err(DomainError::new(
            "idea_link_invalid",
            "Выберите две идеи и допустимый тип связи",
        ));
    }
    let from_book_id = find_idea(state, from_idea_id)?.book_id.as_str();
    let to_book_id = find_idea(state, to_idea_id)?.book_id.as_str();
    if from_book_id != to_book_id {
        return Err(DomainError::new(
            "idea_link_cross_book",
            "Связывать можно только идеи одной книги",
        ));
    }
    Ok(())
}

pub(super) fn apply(
    state: &mut LibraryState,
    action: LibraryAction,
    timestamp: u64,
    make_id: &mut impl FnMut(&str) -> String,
) -> Result<(), DomainError> {
    match action {
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
            let idea = find_idea_mut(state, &idea_id)?;
            if idea.formulation != formulation {
                idea.versions.push(IdeaVersion {
                    formulation: formulation.clone(),
                    saved_at: timestamp,
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
            state.topics.push(Topic {
                id: make_id("topic"),
                name,
            });
        }
        LibraryAction::AssignTopic { idea_id, topic_id } => {
            if !state.topics.iter().any(|item| item.id == topic_id) {
                return Err(DomainError::new("topic_not_found", "Тема не найдена"));
            }
            let idea = find_idea_mut(state, &idea_id)?;
            if !idea.topic_ids.contains(&topic_id) {
                idea.topic_ids.push(topic_id);
            }
        }
        LibraryAction::ConfirmSuggestedTopic { idea_id, name } => {
            find_idea(state, &idea_id)?;
            if name.trim().is_empty() {
                return Err(DomainError::new(
                    "topic_name_required",
                    "Назовите тему знаний",
                ));
            }
            let topic_id = make_id("topic");
            state.topics.push(Topic {
                id: topic_id.clone(),
                name,
            });
            find_idea_mut(state, &idea_id)?.topic_ids.push(topic_id);
        }
        LibraryAction::LinkIdeas {
            from_idea_id,
            to_idea_id,
            relation,
        } => {
            validate_link(state, &from_idea_id, &to_idea_id)?;
            state.idea_links.push(IdeaLink {
                id: make_id("link"),
                from_idea_id,
                to_idea_id,
                relation,
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
            state.materials.push(TransferMaterial {
                id: make_id("material"),
                title,
                problem,
                idea,
                example,
                result,
                limitations,
                idea_ids,
            });
        }
        LibraryAction::RecordReviewResponse {
            idea_id,
            request_kind,
            response,
        } => {
            find_idea(state, &idea_id)?;
            if response.trim().is_empty() {
                return Err(DomainError::new(
                    "review_response_empty",
                    "Codex не вернул текст проверки",
                ));
            }
            state.reviews.retain(|item| {
                item.idea_id != idea_id || item.request_kind != request_kind || !item.pending
            });
            state.reviews.push(IdeaReview {
                id: make_id("review"),
                idea_id,
                request_kind,
                response,
                decision: ReviewDecision::Pending,
                conclusion: String::new(),
                pending: true,
                reviewed_at: timestamp,
            });
        }
        LibraryAction::ResolveReview {
            idea_id,
            request_kind,
            decision,
            formulation,
            conclusion,
        } => {
            find_idea(state, &idea_id)?;
            if decision == ReviewDecision::Pending {
                return Err(DomainError::new(
                    "review_decision_invalid",
                    "Выберите решение по проверке",
                ));
            }
            if decision == ReviewDecision::Refined {
                state.apply_with(
                    LibraryAction::UpdateIdea {
                        idea_id: idea_id.clone(),
                        formulation,
                        assignments: find_idea(state, &idea_id)?.assignments.clone(),
                    },
                    timestamp,
                    make_id,
                )?;
            }
            let existing = state
                .reviews
                .iter()
                .find(|item| {
                    item.idea_id == idea_id && item.request_kind == request_kind && item.pending
                })
                .cloned();
            state.reviews.retain(|item| {
                item.idea_id != idea_id || item.request_kind != request_kind || !item.pending
            });
            state.reviews.push(IdeaReview {
                id: existing
                    .as_ref()
                    .map(|item| item.id.clone())
                    .unwrap_or_else(|| make_id("review")),
                idea_id,
                request_kind,
                response: if decision == ReviewDecision::Later {
                    existing.map(|item| item.response).unwrap_or_default()
                } else {
                    String::new()
                },
                decision,
                conclusion,
                pending: decision == ReviewDecision::Later,
                reviewed_at: timestamp,
            });
        }
        _ => unreachable!("action dispatched to the wrong capability"),
    }
    Ok(())
}
