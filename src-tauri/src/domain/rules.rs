use super::*;

impl LibraryState {
    pub fn debt(&self) -> usize {
        self.drafts.len()
            + self
                .experiments
                .iter()
                .filter(|item| !item.completed)
                .count()
            + self.reviews.iter().filter(|item| item.pending).count()
    }

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
        match action {
            LibraryAction::SaveWorkspaceNote { note } => {
                if note.chars().count() > 240 {
                    return Err(DomainError::new(
                        "workspace_note_too_long",
                        "Пометка не может быть длиннее 240 символов",
                    ));
                }
                self.workspace_note = note;
            }
            LibraryAction::UpdateReading {
                book_id,
                page,
                zoom,
                scroll,
            } => {
                let book = find_book_mut(self, &book_id)?;
                if page == 0 || !(0.5..=4.0).contains(&zoom) {
                    return Err(DomainError::new(
                        "reading_position_invalid",
                        "Проверьте страницу и масштаб",
                    ));
                }
                book.reading = ReadingPosition {
                    page,
                    zoom,
                    scroll: scroll.max(0.0),
                };
            }
            LibraryAction::SaveOutline { book_id, outline } => {
                if outline
                    .iter()
                    .any(|item| item.title.trim().is_empty() || item.page == 0)
                {
                    return Err(DomainError::new(
                        "outline_item_invalid",
                        "Укажите название и страницу раздела",
                    ));
                }
                let ids = outline
                    .iter()
                    .map(|item| item.id.as_str())
                    .collect::<HashSet<_>>();
                let unique_ids = ids.len() == outline.len();
                let parents_exist = outline.iter().all(|item| {
                    item.parent_id
                        .as_deref()
                        .is_none_or(|parent| parent != item.id && ids.contains(parent))
                });
                let has_cycle = outline.iter().any(|item| {
                    let mut seen = HashSet::from([item.id.as_str()]);
                    let mut parent = item.parent_id.as_deref();
                    while let Some(parent_id) = parent {
                        if !seen.insert(parent_id) {
                            return true;
                        }
                        parent = outline
                            .iter()
                            .find(|candidate| candidate.id == parent_id)
                            .and_then(|candidate| candidate.parent_id.as_deref());
                    }
                    false
                });
                if !unique_ids || !parents_exist || has_cycle {
                    return Err(DomainError::new(
                        "outline_structure_invalid",
                        "Проверьте вложенность и уникальность разделов",
                    ));
                }
                find_book_mut(self, &book_id)?.outline = outline;
            }
            LibraryAction::CaptureDraft {
                book_id,
                section,
                page,
                excerpt,
                context,
                comment,
            } => {
                find_book(self, &book_id)?;
                if excerpt.trim().is_empty() || page == 0 {
                    return Err(DomainError::new(
                        "draft_source_required",
                        "Выберите фрагмент и страницу",
                    ));
                }
                self.drafts.push(DraftNote {
                    id: make_id("draft"),
                    book_id,
                    section,
                    page,
                    excerpt,
                    context,
                    comment,
                    created_at: timestamp,
                });
            }
            LibraryAction::ResolveDraftAsIdea {
                draft_id,
                formulation,
                section,
                assignments,
            } => {
                if formulation.trim().is_empty()
                    || section.trim().is_empty()
                    || assignments.is_empty()
                {
                    return Err(DomainError::new(
                        "idea_fields_required",
                        "Нужны авторская формулировка, раздел и хотя бы одно назначение",
                    ));
                }
                let draft = self
                    .drafts
                    .iter()
                    .find(|item| item.id == draft_id)
                    .cloned()
                    .ok_or_else(|| {
                        DomainError::new("draft_not_found", "Черновая заметка не найдена")
                    })?;
                let idea = Idea {
                    id: make_id("idea"),
                    book_id: draft.book_id,
                    section,
                    formulation: formulation.clone(),
                    assignments,
                    fragments: vec![SourceFragment {
                        page: draft.page,
                        excerpt: draft.excerpt,
                        context: draft.context,
                    }],
                    versions: vec![IdeaVersion {
                        formulation,
                        saved_at: timestamp,
                    }],
                    topic_ids: vec![],
                };
                self.ideas.push(idea);
                self.drafts.retain(|item| item.id != draft_id);
            }
            LibraryAction::AttachDraftToIdea { draft_id, idea_id } => {
                let draft = self
                    .drafts
                    .iter()
                    .find(|item| item.id == draft_id)
                    .cloned()
                    .ok_or_else(|| {
                        DomainError::new("draft_not_found", "Черновая заметка не найдена")
                    })?;
                let idea = find_idea_mut(self, &idea_id)?;
                idea.fragments.push(SourceFragment {
                    page: draft.page,
                    excerpt: draft.excerpt,
                    context: draft.context,
                });
                self.drafts.retain(|item| item.id != draft_id);
            }
            LibraryAction::DiscardDraft { draft_id } => {
                if !self.drafts.iter().any(|item| item.id == draft_id) {
                    return Err(DomainError::new(
                        "draft_not_found",
                        "Черновая заметка не найдена",
                    ));
                }
                self.drafts.retain(|item| item.id != draft_id);
            }
            LibraryAction::ActivateStudy { book_id } => {
                find_book(self, &book_id)?;
                self.active_study_book_id = Some(book_id);
            }
            LibraryAction::CompleteReading { book_id } => {
                find_book_mut(self, &book_id)?.reading_completed = true
            }
            LibraryAction::SetStudyRhythm {
                weekly_session_budget,
            } => {
                if !(1..=14).contains(&weekly_session_budget) {
                    return Err(DomainError::new(
                        "study_rhythm_invalid",
                        "Выберите от 1 до 14 сеансов в неделю",
                    ));
                }
                self.weekly_session_budget = weekly_session_budget;
            }
            LibraryAction::SetDebtReminder { days } => {
                if !(1..=90).contains(&days) {
                    return Err(DomainError::new(
                        "debt_reminder_invalid",
                        "Выберите период от 1 до 90 дней",
                    ));
                }
                self.debt_reminder_days = days;
                self.debt_notification_sent_at = None;
            }
            LibraryAction::PlanSession {
                intention,
                planned_at,
            } => {
                if intention.trim().is_empty() {
                    return Err(DomainError::new(
                        "session_intention_required",
                        "Опишите намерение сеанса",
                    ));
                }
                self.sessions.push(StudySession {
                    id: make_id("session"),
                    intention,
                    planned_at,
                    status: SessionStatus::Planned,
                    resolution_reason: String::new(),
                    debt_at_start: 0,
                });
            }
            LibraryAction::StartSession { session_id } => {
                let debt_at_start = self.debt();
                let session = self
                    .sessions
                    .iter_mut()
                    .find(|item| item.id == session_id)
                    .ok_or_else(|| DomainError::new("session_not_found", "Сеанс не найден"))?;
                if session.status != SessionStatus::Planned {
                    return Err(DomainError::new(
                        "session_already_started",
                        "Этот сеанс уже начат или завершён",
                    ));
                }
                session.status = SessionStatus::Active;
                session.debt_at_start = debt_at_start;
            }
            LibraryAction::ResolveSession {
                session_id,
                status,
                reason,
            } => {
                if status != SessionStatus::Completed && reason.trim().is_empty() {
                    return Err(DomainError::new(
                        "session_reason_required",
                        "Укажите причину решения",
                    ));
                }
                let session = self
                    .sessions
                    .iter_mut()
                    .find(|item| item.id == session_id)
                    .ok_or_else(|| DomainError::new("session_not_found", "Сеанс не найден"))?;
                if status == SessionStatus::Completed && session.status != SessionStatus::Active {
                    return Err(DomainError::new(
                        "session_not_started",
                        "Сначала начните сеанс, чтобы измерить изменение долга",
                    ));
                }
                session.status = status;
                session.resolution_reason = reason;
            }
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
                let idea = find_idea_mut(self, &idea_id)?;
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
                self.topics.push(Topic {
                    id: make_id("topic"),
                    name,
                });
            }
            LibraryAction::AssignTopic { idea_id, topic_id } => {
                if !self.topics.iter().any(|item| item.id == topic_id) {
                    return Err(DomainError::new("topic_not_found", "Тема не найдена"));
                }
                let idea = find_idea_mut(self, &idea_id)?;
                if !idea.topic_ids.contains(&topic_id) {
                    idea.topic_ids.push(topic_id);
                }
            }
            LibraryAction::ConfirmSuggestedTopic { idea_id, name } => {
                find_idea(self, &idea_id)?;
                if name.trim().is_empty() {
                    return Err(DomainError::new(
                        "topic_name_required",
                        "Назовите тему знаний",
                    ));
                }
                let topic_id = make_id("topic");
                self.topics.push(Topic {
                    id: topic_id.clone(),
                    name,
                });
                find_idea_mut(self, &idea_id)?.topic_ids.push(topic_id);
            }
            LibraryAction::LinkIdeas {
                from_idea_id,
                to_idea_id,
                relation,
            } => {
                if from_idea_id == to_idea_id {
                    return Err(DomainError::new(
                        "idea_link_invalid",
                        "Выберите две идеи и допустимый тип связи",
                    ));
                }
                find_idea(self, &from_idea_id)?;
                find_idea(self, &to_idea_id)?;
                self.idea_links.push(IdeaLink {
                    id: make_id("link"),
                    from_idea_id,
                    to_idea_id,
                    relation,
                });
            }
            LibraryAction::CompleteExperiment {
                idea_id,
                situation,
                action,
                result,
                conclusion,
                successful,
            } => {
                find_idea(self, &idea_id)?;
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
                self.experiments.push(Experiment {
                    id: make_id("experiment"),
                    idea_id,
                    situation,
                    action,
                    result,
                    conclusion,
                    successful,
                    completed: true,
                });
            }
            LibraryAction::CompleteRecall {
                idea_id,
                answer,
                rating,
                next_at,
            } => {
                find_idea(self, &idea_id)?;
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
                self.recalls.push(Recall {
                    id: make_id("recall"),
                    idea_id,
                    answer,
                    rating,
                    next_at: next_at.unwrap_or_else(|| timestamp + suggested_days * 86_400),
                });
            }
            LibraryAction::RescheduleRecall { recall_id, next_at } => {
                if next_at == 0 {
                    return Err(DomainError::new(
                        "recall_schedule_invalid",
                        "Выберите дату следующего восстановления",
                    ));
                }
                let recall = self
                    .recalls
                    .iter_mut()
                    .find(|item| item.id == recall_id)
                    .ok_or_else(|| {
                        DomainError::new("recall_not_found", "Восстановление не найдено")
                    })?;
                recall.next_at = next_at;
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
                self.materials.push(TransferMaterial {
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
                find_idea(self, &idea_id)?;
                if response.trim().is_empty() {
                    return Err(DomainError::new(
                        "review_response_empty",
                        "Codex не вернул текст проверки",
                    ));
                }
                self.reviews.retain(|item| {
                    item.idea_id != idea_id || item.request_kind != request_kind || !item.pending
                });
                self.reviews.push(IdeaReview {
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
                find_idea(self, &idea_id)?;
                if decision == ReviewDecision::Pending {
                    return Err(DomainError::new(
                        "review_decision_invalid",
                        "Выберите решение по проверке",
                    ));
                }
                if decision == ReviewDecision::Refined {
                    self.apply_with(
                        LibraryAction::UpdateIdea {
                            idea_id: idea_id.clone(),
                            formulation,
                            assignments: find_idea(self, &idea_id)?.assignments.clone(),
                        },
                        timestamp,
                        make_id,
                    )?;
                }
                let existing = self
                    .reviews
                    .iter()
                    .find(|item| {
                        item.idea_id == idea_id && item.request_kind == request_kind && item.pending
                    })
                    .cloned();
                self.reviews.retain(|item| {
                    item.idea_id != idea_id || item.request_kind != request_kind || !item.pending
                });
                self.reviews.push(IdeaReview {
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
            LibraryAction::CompleteStudy {
                book_id,
                retrospective,
                significant_idea_ids,
                continuing_work,
                debt_decision,
            } => {
                find_book(self, &book_id)?;
                if retrospective.trim().is_empty()
                    || !(3..=7).contains(&significant_idea_ids.len())
                    || debt_decision.trim().is_empty()
                {
                    return Err(DomainError::new(
                        "retrospective_required",
                        "Добавьте ретроспективу, 3–7 значимых идей и решение по долгу",
                    ));
                }
                if significant_idea_ids.iter().any(|id| {
                    !self
                        .ideas
                        .iter()
                        .any(|idea| &idea.id == id && idea.book_id == book_id)
                }) {
                    return Err(DomainError::new(
                        "retrospective_ideas_invalid",
                        "Значимые идеи должны относиться к этой книге",
                    ));
                }
                let book = find_book_mut(self, &book_id)?;
                book.study_completed = true;
                book.retrospective = Some(Retrospective {
                    text: retrospective,
                    significant_idea_ids,
                    continuing_work,
                    debt_decision,
                });
                if self.active_study_book_id.as_deref() == Some(book_id.as_str()) {
                    self.active_study_book_id = None;
                }
            }
        }
        Ok(())
    }
}
