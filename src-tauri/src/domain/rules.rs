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
                if page > book.farthest_page {
                    book.farthest_page = page;
                    self.milestones.push(StudyMilestone {
                        id: make_id("milestone"),
                        book_id,
                        kind: MilestoneKind::ReadingProgress,
                        occurred_at: timestamp,
                        page: Some(page),
                    });
                }
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
                self.milestones.push(StudyMilestone {
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
                find_book(self, &book_id)?;
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
                self.drafts.push(DraftNote {
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
                self.milestones.push(StudyMilestone {
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
                self.ideas.push(idea);
                self.drafts.retain(|item| item.id != draft_id);
                self.milestones.push(StudyMilestone {
                    id: make_id("milestone"),
                    book_id: self
                        .ideas
                        .last()
                        .map(|idea| idea.book_id.clone())
                        .unwrap_or_default(),
                    kind: MilestoneKind::IdeaFormulated,
                    occurred_at: timestamp,
                    page: None,
                });
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
                if draft.fragments.is_empty() {
                    idea.fragments.push(SourceFragment {
                        page: draft.page,
                        excerpt: draft.excerpt,
                        context: draft.context,
                    });
                } else {
                    idea.fragments.extend(draft.fragments);
                }
                self.drafts.retain(|item| item.id != draft_id);
            }
            LibraryAction::DeferDraft { draft_id } => {
                let index = self
                    .drafts
                    .iter()
                    .position(|item| item.id == draft_id)
                    .ok_or_else(|| {
                        DomainError::new("draft_not_found", "Черновая заметка не найдена")
                    })?;
                let draft = self.drafts.remove(index);
                self.drafts.push(draft);
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
                if let Some(active_id) = self.active_study_book_id.clone() {
                    if active_id != book_id {
                        find_book_mut(self, &active_id)?.study_status = StudyStatus::Paused;
                    }
                }
                let book = find_book_mut(self, &book_id)?;
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
                self.active_study_book_id = Some(book_id);
            }
            LibraryAction::CompleteReading { book_id } => {
                let book = find_book_mut(self, &book_id)?;
                book.reading_completed = true;
                book.study_status = StudyStatus::ReadyToComplete;
            }
            LibraryAction::ArchiveBook { book_id } => {
                let was_active = self.active_study_book_id.as_deref() == Some(book_id.as_str());
                let book = find_book_mut(self, &book_id)?;
                book.archived = true;
                if was_active {
                    book.study_status = StudyStatus::Paused;
                    self.active_study_book_id = None;
                }
            }
            LibraryAction::RestoreBook { book_id } => {
                let book = find_book_mut(self, &book_id)?;
                book.archived = false;
            }
            LibraryAction::StartRepeatStudy { book_id } => {
                if find_book(self, &book_id)?.study_status != StudyStatus::Completed {
                    return Err(DomainError::new(
                        "repeat_study_requires_completion",
                        "Повторное изучение доступно после завершённого цикла",
                    ));
                }
                if let Some(active_id) = self.active_study_book_id.clone() {
                    if active_id != book_id {
                        find_book_mut(self, &active_id)?.study_status = StudyStatus::Paused;
                    }
                }
                let book = find_book_mut(self, &book_id)?;
                book.reading_completed = false;
                book.study_status = StudyStatus::Repeating;
                book.study_cycles.push(StudyCycle {
                    id: make_id("study-cycle"),
                    started_at: timestamp,
                    completed_at: None,
                    retrospective: None,
                });
                self.active_study_book_id = Some(book_id);
            }
            LibraryAction::UpdateReaderPreferences {
                book_id,
                preferences,
            } => {
                if !(320..=560).contains(&preferences.sidebar_width) {
                    return Err(DomainError::new(
                        "reader_sidebar_width_invalid",
                        "Ширина панели должна быть от 320 до 560 пикселей",
                    ));
                }
                find_book_mut(self, &book_id)?.reader = preferences;
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
            } => {
                let book_id = find_idea(self, &idea_id)?.book_id.clone();
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
                    status: ExperimentStatus::Completed,
                    cancellation_reason: String::new(),
                    next_step: String::new(),
                });
                self.milestones.push(StudyMilestone {
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
                let idea_id = {
                    let experiment = self
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
                let book_id = find_idea(self, &idea_id)?.book_id.clone();
                self.milestones.push(StudyMilestone {
                    id: make_id("milestone"),
                    book_id,
                    kind: MilestoneKind::ExperimentAdvanced,
                    occurred_at: timestamp,
                    page: None,
                });
            }
            LibraryAction::CompleteRecall {
                idea_id,
                answer,
                rating,
                next_at,
            } => {
                let book_id = find_idea(self, &idea_id)?.book_id.clone();
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
                self.milestones.push(StudyMilestone {
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
                unfinished_work_decision,
            } => {
                find_book(self, &book_id)?;
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
                book.study_status = StudyStatus::Completed;
                let completed_retrospective = Retrospective {
                    text: retrospective,
                    significant_idea_ids,
                    continuing_work,
                    unfinished_work_decision,
                };
                book.retrospective = Some(completed_retrospective.clone());
                if let Some(cycle) = book.study_cycles.last_mut() {
                    cycle.completed_at = Some(timestamp);
                    cycle.retrospective = Some(completed_retrospective);
                }
                if self.active_study_book_id.as_deref() == Some(book_id.as_str()) {
                    self.active_study_book_id = None;
                }
                self.completion_drafts
                    .retain(|draft| draft.book_id != book_id);
                self.milestones.push(StudyMilestone {
                    id: make_id("milestone"),
                    book_id,
                    kind: MilestoneKind::StudyCompleted,
                    occurred_at: timestamp,
                    page: None,
                });
            }
            LibraryAction::SaveStudyCompletionDraft { draft } => {
                find_book(self, &draft.book_id)?;
                if !(1..=6).contains(&draft.step) {
                    return Err(DomainError::new(
                        "completion_step_invalid",
                        "Шаг завершения должен быть от 1 до 6",
                    ));
                }
                self.completion_drafts
                    .retain(|existing| existing.book_id != draft.book_id);
                self.completion_drafts.push(draft);
            }
        }
        Ok(())
    }
}
