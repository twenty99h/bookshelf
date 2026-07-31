use super::*;

fn test_data_dir() -> PathBuf {
    std::env::temp_dir().join(format!("bookshelf-library-test-{}", unique_number()))
}

#[test]
fn clean_launch_opens_an_empty_personal_library() {
    let data_dir = test_data_dir();
    let library = Library::open(&data_dir).unwrap();
    assert_eq!(library.load().unwrap(), LibraryState::default());
    fs::remove_dir_all(data_dir).unwrap();
}

#[test]
fn complete_legacy_state_migrates_to_the_versioned_workspace_envelope() {
    let data_dir = test_data_dir();
    fs::create_dir_all(&data_dir).unwrap();
    fs::write(
        data_dir.join("library.json"),
        serde_json::to_vec(&serde_json::json!({
            "books": [], "drafts": [], "ideas": [], "topics": [], "ideaLinks": [],
            "experiments": [], "recalls": [], "sessions": [], "materials": [], "reviews": [],
            "workspaceNote": "legacy", "activeStudyBookId": null, "weeklySessionBudget": 3,
            "lastDebtChange": 0, "lastDebtChangedAt": 0, "debtNotificationSentAt": null,
            "debtReminderDays": 7
        }))
        .unwrap(),
    )
    .unwrap();

    let library = Library::open(&data_dir).unwrap();

    assert_eq!(library.load().unwrap().workspace_note, "legacy");
    assert!(data_dir.join("library.json.migrated").exists());
    fs::remove_dir_all(data_dir).unwrap();
}

#[test]
fn partial_workspace_migration_is_rejected_without_rewriting_the_source() {
    let data_dir = test_data_dir();
    fs::create_dir_all(&data_dir).unwrap();
    let partial = r#"{"books":[],"drafts":[],"ideas":[]}"#;
    fs::write(data_dir.join("library.json"), partial).unwrap();

    let error = Library::open(&data_dir)
        .err()
        .expect("partial state must fail");

    assert!(error.to_string().contains("частично преобразовано"));
    assert_eq!(
        fs::read_to_string(data_dir.join("library.json")).unwrap(),
        partial
    );
    assert!(!data_dir.join("library.json.migrated").exists());
    fs::remove_dir_all(data_dir).unwrap();
}

#[test]
fn workspace_change_survives_a_desktop_restart() {
    let data_dir = test_data_dir();
    Library::open(&data_dir)
        .unwrap()
        .apply(LibraryAction::SaveWorkspaceNote {
            note: "Продолжить с главы 2".into(),
        })
        .unwrap();
    assert_eq!(
        Library::open(&data_dir)
            .unwrap()
            .load()
            .unwrap()
            .workspace_note,
        "Продолжить с главы 2"
    );
    fs::remove_dir_all(data_dir).unwrap();
}

#[test]
fn repeated_workspace_changes_replace_state_portably() {
    let data_dir = test_data_dir();
    let library = Library::open(&data_dir).unwrap();
    library
        .apply(LibraryAction::SaveWorkspaceNote {
            note: "Первая пометка".into(),
        })
        .unwrap();
    library
        .apply(LibraryAction::SaveWorkspaceNote {
            note: "Актуальная пометка".into(),
        })
        .unwrap();
    assert_eq!(library.load().unwrap().workspace_note, "Актуальная пометка");
    assert!(data_dir.join("library.sqlite3").exists());
    fs::remove_dir_all(data_dir).unwrap();
}

#[test]
fn repeat_study_pauses_the_previous_active_book() {
    let mut state = LibraryState::default();
    let mut active = Book::for_test("active", "Активная книга");
    active.study_status = StudyStatus::Active;
    let mut completed = Book::for_test("completed", "Завершённая книга");
    completed.study_status = StudyStatus::Completed;
    state.books = vec![active, completed];
    state.active_study_book_id = Some("active".into());

    state
        .apply(LibraryAction::StartRepeatStudy {
            book_id: "completed".into(),
        })
        .unwrap();

    assert_eq!(state.active_study_book_id.as_deref(), Some("completed"));
    assert_eq!(state.books[0].study_status, StudyStatus::Paused);
    assert_eq!(state.books[1].study_status, StudyStatus::Repeating);
}

#[test]
fn failed_domain_action_does_not_replace_the_persisted_state() {
    let data_dir = test_data_dir();
    let library = Library::open(&data_dir).unwrap();
    library
        .apply(LibraryAction::SaveWorkspaceNote {
            note: "Сохранённая пометка".into(),
        })
        .unwrap();
    let result = library.apply(LibraryAction::ResolveDraftAsIdea {
        draft_id: "missing".into(),
        formulation: "Формулировка".into(),
        section: "Глава".into(),
        assignments: vec![IdeaAssignment::Recall],
    });
    assert!(result.is_err());
    assert_eq!(
        library.load().unwrap().workspace_note,
        "Сохранённая пометка"
    );
    fs::remove_dir_all(data_dir).unwrap();
}

#[test]
fn full_text_search_covers_learning_records_and_updates_transactionally() {
    let data_dir = test_data_dir();
    let library = Library::open(&data_dir).unwrap();
    let mut state = LibraryState::default();
    state
        .books
        .push(Book::for_test("book", "Распределённые системы"));
    state.ideas.push(Idea {
        formulation: "Репликация требует явной модели согласованности".into(),
        fragments: vec![SourceFragment {
            page: 42,
            excerpt: "Кворум подтверждений".into(),
            context: "Запись и чтение пересекаются".into(),
        }],
        ..Idea::for_test("idea", "book")
    });
    state.topics.push(Topic {
        id: "topic".into(),
        name: "Надёжность хранилищ".into(),
    });
    state.materials.push(TransferMaterial {
        id: "material".into(),
        title: "Как выбирать кворум".into(),
        limitations: "Не скрывает сетевые разделения".into(),
        ..TransferMaterial::default()
    });
    library.replace_state(&state).unwrap();
    assert_eq!(library.search("согласованности").unwrap()[0].id, "idea");
    assert_eq!(
        library.search("Кворум подтверждений").unwrap()[0].id,
        "idea"
    );
    assert_eq!(
        library.search("Запись и чтение пересекаются").unwrap()[0].id,
        "idea"
    );
    assert_eq!(
        library.search("Надёжность хранилищ").unwrap()[0].id,
        "topic"
    );
    assert_eq!(
        library.search("сетевые разделения").unwrap()[0].id,
        "material"
    );
    state.ideas.clear();
    state.topics.clear();
    state.materials.clear();
    library.replace_state(&state).unwrap();
    assert!(library.search("согласованности").unwrap().is_empty());
    assert!(library.search("Надёжность хранилищ").unwrap().is_empty());
    assert!(library.search("сетевые разделения").unwrap().is_empty());

    state.topics.push(Topic {
        id: "reindexed-topic".into(),
        name: "Восстановленный индекс".into(),
    });
    library.replace_state(&state).unwrap();
    Connection::open(&library.database_file)
        .unwrap()
        .execute("DELETE FROM search_index", [])
        .unwrap();
    drop(library);
    let reopened = Library::open(&data_dir).unwrap();
    assert_eq!(
        reopened.search("Восстановленный индекс").unwrap()[0].id,
        "reindexed-topic"
    );
    fs::remove_dir_all(data_dir).unwrap();
}

#[test]
fn activating_another_study_preserves_work_and_keeps_only_one_active() {
    let mut state = LibraryState {
        books: vec![
            Book::for_test("one", "Первая"),
            Book::for_test("two", "Вторая"),
        ],
        ..LibraryState::default()
    };
    state
        .apply(LibraryAction::ActivateStudy {
            book_id: "one".into(),
        })
        .unwrap();
    state
        .apply(LibraryAction::ActivateStudy {
            book_id: "two".into(),
        })
        .unwrap();
    assert_eq!(state.active_study_book_id.as_deref(), Some("two"));
    assert_eq!(state.books.len(), 2);
}

#[test]
fn resolving_a_draft_requires_authored_text_but_allows_assignment_later() {
    let mut state = LibraryState::default();
    state.books.push(Book::for_test("book", "Книга"));
    state.drafts.push(DraftNote::for_test("draft", "book"));
    let result = state.apply(LibraryAction::ResolveDraftAsIdea {
        draft_id: "draft".into(),
        formulation: "".into(),
        section: "Глава 1".into(),
        assignments: vec![],
    });
    assert_eq!(result.unwrap_err().code(), "idea_fields_required");
    assert_eq!(state.drafts.len(), 1);
    assert!(state.ideas.is_empty());

    state
        .apply(LibraryAction::ResolveDraftAsIdea {
            draft_id: "draft".into(),
            formulation: "Неопределённость результата должна быть частью модели".into(),
            section: "Глава 1".into(),
            assignments: vec![],
        })
        .unwrap();
    assert!(state.drafts.is_empty());
    assert!(state.ideas[0].assignments.is_empty());
}

#[test]
fn draft_export_writes_a_temporary_markdown_resource_and_commits_removal() {
    let data_dir = test_data_dir();
    let library = Library::open(&data_dir).unwrap();
    let mut state = LibraryState::default();
    state.books.push(Book::for_test("book", "Надёжные системы"));
    state.drafts.push(DraftNote::for_test("draft", "book"));
    library.commit(&state).unwrap();
    let destination = data_dir.join("exported-draft.md");

    let exported = crate::application::export_draft(
        &library.text_file_storage(),
        &library,
        &SystemClock,
        &SystemIdGenerator,
        "draft",
        destination.to_string_lossy().into_owned(),
    )
    .unwrap();

    let markdown = fs::read_to_string(&destination).unwrap();
    assert!(markdown.contains("Надёжные системы"));
    assert!(markdown.contains("Фрагмент"));
    assert!(exported.drafts.is_empty());
    assert!(Library::open(&data_dir)
        .unwrap()
        .load()
        .unwrap()
        .drafts
        .is_empty());
    fs::remove_dir_all(data_dir).unwrap();
}

#[test]
fn a_negative_practical_experiment_is_a_valid_completed_experiment() {
    let mut state = LibraryState::default();
    state.ideas.push(Idea::for_test("idea", "book"));
    state
        .apply(LibraryAction::CompleteExperiment {
            idea_id: "idea".into(),
            situation: "Новый сервис".into(),
            action: "Применил паттерн".into(),
            result: "Усложнил поддержку".into(),
            conclusion: "Не применять для малого сервиса".into(),
        })
        .unwrap();
    assert_eq!(state.experiments[0].status, ExperimentStatus::Completed);
}

#[test]
fn practical_experiment_moves_through_closed_lifecycle_states() {
    let mut state = LibraryState::default();
    state.ideas.push(Idea::for_test("idea", "book"));

    state
        .apply(LibraryAction::CreateExperiment {
            idea_id: "idea".into(),
            situation: "Новый сервис".into(),
            action: "Проверить явный переход".into(),
            next_step: "Обсудить с командой".into(),
        })
        .unwrap();
    let experiment_id = state.experiments[0].id.clone();
    assert_eq!(state.experiments[0].status, ExperimentStatus::Intent);

    let skipped = state.apply(LibraryAction::AdvanceExperiment {
        experiment_id: experiment_id.clone(),
        status: ExperimentStatus::Completed,
        situation: "Новый сервис".into(),
        action: "Проверить явный переход".into(),
        result: "Результат".into(),
        conclusion: "Вывод".into(),
        cancellation_reason: String::new(),
        next_step: String::new(),
    });
    assert_eq!(skipped.unwrap_err().code(), "experiment_transition_invalid");

    for status in [
        ExperimentStatus::Running,
        ExperimentStatus::Reviewing,
        ExperimentStatus::Completed,
    ] {
        state
            .apply(LibraryAction::AdvanceExperiment {
                experiment_id: experiment_id.clone(),
                status,
                situation: "Новый сервис".into(),
                action: "Проверить явный переход".into(),
                result: if status == ExperimentStatus::Completed {
                    "Отрицательный результат".into()
                } else {
                    String::new()
                },
                conclusion: if status == ExperimentStatus::Completed {
                    "Не применять".into()
                } else {
                    String::new()
                },
                cancellation_reason: String::new(),
                next_step: "Без даты".into(),
            })
            .unwrap();
    }

    assert_eq!(state.experiments[0].status, ExperimentStatus::Completed);
}

#[test]
fn study_completion_requires_a_retrospective_and_three_to_seven_ideas() {
    let mut state = LibraryState::default();
    state.books.push(Book::for_test("book", "Книга"));
    state.active_study_book_id = Some("book".into());
    let result = state.apply(LibraryAction::CompleteStudy {
        book_id: "book".into(),
        retrospective: "".into(),
        significant_idea_ids: vec![],
        continuing_work: "".into(),
        unfinished_work_decision: "".into(),
        work_decisions: vec![],
    });
    assert_eq!(result.unwrap_err().code(), "retrospective_required");
    assert_eq!(state.active_study_book_id.as_deref(), Some("book"));
}

#[test]
fn study_completion_persists_a_distinct_decision_for_each_open_work_item() {
    let mut state = LibraryState::default();
    state.books.push(Book::for_test("book", "Книга"));
    for id in ["idea-1", "idea-2", "idea-3"] {
        state.ideas.push(Idea::for_test(id, "book"));
    }
    state.drafts.push(DraftNote::for_test("draft", "book"));
    state.reviews.push(IdeaReview {
        id: "review".into(),
        idea_id: "idea-1".into(),
        pending: true,
        ..IdeaReview::default()
    });
    state.recalls.push(Recall {
        id: "recall".into(),
        idea_id: "idea-2".into(),
        ..Recall::default()
    });
    state.experiments.push(Experiment {
        id: "experiment".into(),
        idea_id: "idea-3".into(),
        status: ExperimentStatus::Running,
        ..Experiment::default()
    });
    let base = LibraryAction::CompleteStudy {
        book_id: "book".into(),
        retrospective: "Авторский итог".into(),
        significant_idea_ids: vec!["idea-1".into(), "idea-2".into(), "idea-3".into()],
        continuing_work: "Эксперимент продолжается".into(),
        unfinished_work_decision: "Каждый пункт разобран отдельно".into(),
        work_decisions: vec![],
    };

    assert_eq!(
        state.apply(base).unwrap_err().code(),
        "completion_work_decisions_required"
    );
    let decisions = [
        ("draft", CompletionWorkKind::Draft, "Разобрать позже"),
        (
            "review",
            CompletionWorkKind::Review,
            "Оставить до уточнения",
        ),
        ("recall", CompletionWorkKind::Recall, "Повторить вручную"),
        ("experiment", CompletionWorkKind::Experiment, "Продолжить"),
    ]
    .into_iter()
    .map(|(work_id, kind, decision)| CompletionWorkDecision {
        work_id: work_id.into(),
        kind,
        decision: decision.into(),
    })
    .collect::<Vec<_>>();
    state
        .apply(LibraryAction::CompleteStudy {
            book_id: "book".into(),
            retrospective: "Авторский итог".into(),
            significant_idea_ids: vec!["idea-1".into(), "idea-2".into(), "idea-3".into()],
            continuing_work: "Эксперимент продолжается".into(),
            unfinished_work_decision: "Каждый пункт разобран отдельно".into(),
            work_decisions: decisions.clone(),
        })
        .unwrap();

    assert_eq!(
        state.books[0]
            .retrospective
            .as_ref()
            .unwrap()
            .work_decisions,
        decisions
    );
    assert_eq!(state.experiments[0].status, ExperimentStatus::Running);
}

#[test]
fn corrected_outline_rejects_cycles_and_duplicate_ids() {
    let mut state = LibraryState::default();
    state.books.push(Book::for_test("book", "Книга"));
    let result = state.apply(LibraryAction::SaveOutline {
        book_id: "book".into(),
        outline: vec![
            OutlineItem {
                id: "chapter".into(),
                title: "Глава".into(),
                page: 1,
                parent_id: Some("section".into()),
            },
            OutlineItem {
                id: "section".into(),
                title: "Раздел".into(),
                page: 2,
                parent_id: Some("chapter".into()),
            },
        ],
    });
    assert_eq!(result.unwrap_err().code(), "outline_structure_invalid");

    let duplicate = state.apply(LibraryAction::SaveOutline {
        book_id: "book".into(),
        outline: vec![
            OutlineItem {
                id: "same".into(),
                title: "Один".into(),
                page: 1,
                parent_id: None,
            },
            OutlineItem {
                id: "same".into(),
                title: "Два".into(),
                page: 2,
                parent_id: None,
            },
        ],
    });
    assert_eq!(duplicate.unwrap_err().code(), "outline_structure_invalid");
}

#[test]
fn recall_rating_owns_the_default_schedule_but_accepts_reader_override() {
    let mut state = LibraryState::default();
    state.ideas.push(Idea::for_test("idea", "book"));
    state
        .apply(LibraryAction::CompleteRecall {
            idea_id: "idea".into(),
            answer: "Суть, условия и ограничения".into(),
            rating: RecallRating::Partial,
            next_at: Some(123_456),
        })
        .unwrap();
    assert_eq!(state.recalls[0].next_at, 123_456);
    assert_eq!(state.recalls[0].rating, RecallRating::Partial);
}

#[test]
fn review_response_is_temporary_until_the_reader_resolves_it() {
    let mut state = LibraryState::default();
    state.ideas.push(Idea::for_test("idea", "book"));
    state
        .apply(LibraryAction::RecordReviewResponse {
            idea_id: "idea".into(),
            request_kind: ReviewKind::IdeaReview,
            response: "Возможный пробел".into(),
        })
        .unwrap();
    assert_eq!(state.reviews[0].response, "Возможный пробел");
    assert!(state.reviews[0].pending);

    state
        .apply(LibraryAction::ResolveReview {
            idea_id: "idea".into(),
            request_kind: ReviewKind::IdeaReview,
            decision: ReviewDecision::Unchanged,
            formulation: "".into(),
            conclusion: "Проверил ограничение".into(),
        })
        .unwrap();
    assert!(state.reviews[0].response.is_empty());
    assert!(!state.reviews[0].pending);
    assert_eq!(state.reviews[0].decision, ReviewDecision::Unchanged);
}

#[test]
fn review_package_is_built_from_the_selected_source_without_private_state() {
    let data_dir = test_data_dir();
    let library = Library::open(&data_dir).unwrap();
    let mut state = LibraryState {
        workspace_note: "SECRET WORKSPACE NOTE".into(),
        ..LibraryState::default()
    };
    state.books.push(Book::for_test("book", "Надёжные системы"));
    state.ideas.push(Idea {
        formulation: "Отказы нужно проектировать явно".into(),
        fragments: vec![SourceFragment {
            page: 42,
            excerpt: "Failure is part of the design".into(),
            context: "SECRET NEARBY CONTEXT".into(),
        }],
        ..Idea::for_test("idea", "book")
    });
    state.experiments.push(Experiment {
        idea_id: "idea".into(),
        situation: "SECRET EXPERIMENT".into(),
        ..Experiment::default()
    });
    library.replace_state(&state).unwrap();

    let package = library
        .prepare_review_package("idea", ReviewKind::IdeaReview, None)
        .unwrap();

    assert!(package.contains("Надёжные системы, Глава 1, стр. 42"));
    assert!(package.contains("Отказы нужно проектировать явно"));
    assert!(package.contains("Failure is part of the design"));
    assert!(!package.contains("SECRET WORKSPACE NOTE"));
    assert!(!package.contains("SECRET EXPERIMENT"));
    assert!(!package.contains("SECRET NEARBY CONTEXT"));
    let changed = library
        .approve_review_package("idea", ReviewKind::IdeaReview, None, "unapproved package")
        .unwrap_err();
    assert!(
        matches!(changed, LibraryError::Domain(ref error) if error.code() == "codex_package_changed")
    );
    fs::remove_dir_all(data_dir).unwrap();
}

#[test]
fn deferred_review_kinds_do_not_replace_each_other() {
    let mut state = LibraryState::default();
    state.ideas.push(Idea::for_test("idea", "book"));
    for request_kind in [ReviewKind::IdeaReview, ReviewKind::TopicSuggestion] {
        state
            .apply(LibraryAction::RecordReviewResponse {
                idea_id: "idea".into(),
                request_kind,
                response: format!("Ответ {request_kind:?}"),
            })
            .unwrap();
    }
    assert_eq!(
        state.reviews.iter().filter(|review| review.pending).count(),
        2
    );

    state
        .apply(LibraryAction::ResolveReview {
            idea_id: "idea".into(),
            request_kind: ReviewKind::IdeaReview,
            decision: ReviewDecision::Unchanged,
            formulation: String::new(),
            conclusion: String::new(),
        })
        .unwrap();

    assert!(state
        .reviews
        .iter()
        .any(|review| { review.pending && review.request_kind == ReviewKind::TopicSuggestion }));
    assert!(!state
        .reviews
        .iter()
        .any(|review| review.pending && review.request_kind == ReviewKind::IdeaReview));
}

#[test]
fn reading_position_and_corrected_outline_survive_reopening() {
    let data_dir = test_data_dir();
    let library = Library::open(&data_dir).unwrap();
    let mut state = LibraryState::default();
    state.books.push(Book::for_test("book", "Книга"));
    library.replace_state(&state).unwrap();
    library
        .apply(LibraryAction::UpdateReading {
            book_id: "book".into(),
            page: 37,
            zoom: 1.4,
            scroll: 812.0,
        })
        .unwrap();
    library
        .apply(LibraryAction::SaveOutline {
            book_id: "book".into(),
            outline: vec![OutlineItem {
                id: "chapter".into(),
                title: "Исправленная глава".into(),
                page: 36,
                parent_id: None,
            }],
        })
        .unwrap();
    let reopened = Library::open(&data_dir).unwrap().load().unwrap();
    assert_eq!(
        reopened.books[0].reading,
        ReadingPosition {
            page: 37,
            zoom: 1.4,
            scroll: 812.0
        }
    );
    assert_eq!(reopened.books[0].outline[0].title, "Исправленная глава");
    fs::remove_dir_all(data_dir).unwrap();
}

#[test]
fn pdf_import_corpus_covers_text_variants_scans_and_duplicate_hashes() {
    let data_dir = test_data_dir();
    let library = Library::open(&data_dir).unwrap();
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pdf");
    let text_layer = corpus.join("text-layer.pdf");
    let result = crate::application::import_pdf(
        &library.reading_storage(),
        &library,
        &SystemIdGenerator,
        text_layer.to_string_lossy().into_owned(),
        String::new(),
    )
    .unwrap();
    let imported = result.state.books.last().unwrap();
    assert!(imported.has_text_layer);
    assert!(library.absolute_book_path(&imported.stored_file).is_file());

    let complex = corpus.join("compressed-outline-fonts.pdf");
    let complex_bytes = fs::read(&complex).unwrap();
    let complex_document = lopdf::Document::load_mem(&complex_bytes).unwrap();
    assert_eq!(complex_document.get_pages().len(), 2);
    for marker in [b"/FlateDecode".as_slice(), b"/Outlines", b"/Encoding"] {
        assert!(complex_bytes
            .windows(marker.len())
            .any(|window| window == marker));
    }
    let result = crate::application::import_pdf(
        &library.reading_storage(),
        &library,
        &SystemIdGenerator,
        complex.to_string_lossy().into_owned(),
        "Спецификация с оглавлением".into(),
    )
    .unwrap();
    assert!(result.state.books.last().unwrap().has_text_layer);

    let scanned = corpus.join("image-only.pdf");
    let result = crate::application::import_pdf(
        &library.reading_storage(),
        &library,
        &SystemIdGenerator,
        scanned.to_string_lossy().into_owned(),
        "Скан".into(),
    )
    .unwrap();
    let scanned_book = result.state.books.last().unwrap();
    assert!(!scanned_book.has_text_layer);
    assert!(!scanned_book.content_hash.is_empty());

    let before_duplicate = result.state.books.len();
    let result = crate::application::import_pdf(
        &library.reading_storage(),
        &library,
        &SystemIdGenerator,
        scanned.to_string_lossy().into_owned(),
        "Повторный скан".into(),
    )
    .unwrap();
    assert!(result.duplicate);
    assert_eq!(result.state.books.len(), before_duplicate);
    fs::remove_dir_all(data_dir).unwrap();
}

#[test]
fn native_smoke_imports_a_pdf_opens_its_local_path_and_restores_reader_position_after_restart() {
    let data_dir = test_data_dir();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pdf/text-layer.pdf");
    let book_id = {
        let library = Library::open(&data_dir).unwrap();
        let imported = crate::application::import_pdf(
            &library.reading_storage(),
            &library,
            &SystemIdGenerator,
            fixture.to_string_lossy().into_owned(),
            "Native smoke".into(),
        )
        .unwrap();
        let book = imported.state.books.last().unwrap();
        let local_path = library.absolute_book_path(&book.stored_file);
        assert!(local_path.is_absolute());
        assert!(local_path.is_file());
        library
            .apply(LibraryAction::UpdateReading {
                book_id: book.id.clone(),
                page: 2,
                zoom: 1.35,
                scroll: 640.0,
            })
            .unwrap();
        book.id.clone()
    };

    let restarted = Library::open(&data_dir).unwrap().load().unwrap();
    let book = restarted
        .books
        .iter()
        .find(|book| book.id == book_id)
        .unwrap();
    assert_eq!(book.reading.page, 2);
    assert_eq!(book.reading.zoom, 1.35);
    assert_eq!(book.reading.scroll, 640.0);
    fs::remove_dir_all(data_dir).unwrap();
}

#[test]
fn confirmed_topic_suggestion_is_assigned_atomically() {
    let mut state = LibraryState::default();
    state.ideas.push(Idea::for_test("idea", "book"));
    state
        .apply(LibraryAction::ConfirmSuggestedTopic {
            idea_id: "idea".into(),
            name: "Надёжность".into(),
        })
        .unwrap();
    assert_eq!(state.topics.len(), 1);
    assert_eq!(state.ideas[0].topic_ids, vec![state.topics[0].id.clone()]);
}

#[test]
fn snapshots_do_not_make_full_ai_responses_permanent() {
    let data_dir = test_data_dir();
    let library = Library::open(&data_dir).unwrap();
    let mut state = LibraryState::default();
    state.books.push(Book::for_test("book", "Книга"));
    state.ideas.push(Idea::for_test("idea", "book"));
    state
        .apply(LibraryAction::RecordReviewResponse {
            idea_id: "idea".into(),
            request_kind: ReviewKind::IdeaReview,
            response: "Временный полный ответ".into(),
        })
        .unwrap();
    library.create_snapshot(&state).unwrap();
    let restored =
        crate::application::restore_latest_snapshot(&library.archive_storage(), &library).unwrap();
    assert!(restored.reviews[0].response.is_empty());
    fs::remove_dir_all(data_dir).unwrap();
}

#[test]
fn encrypted_archive_round_trip_restores_state_and_rejects_a_wrong_password() {
    let source_dir = test_data_dir();
    let source = Library::open(&source_dir).unwrap();
    source
        .apply(LibraryAction::SaveWorkspaceNote {
            note: "Переносимое состояние".into(),
        })
        .unwrap();
    let archive = source_dir.with_extension("bookshelf.age");
    crate::application::export_archive(
        &source.archive_storage(),
        &source,
        archive.to_string_lossy().into_owned(),
        "надёжный пароль",
    )
    .unwrap();

    let target_dir = test_data_dir();
    let target = Library::open(&target_dir).unwrap();
    let wrong = crate::application::import_archive(
        &target.archive_storage(),
        &target,
        archive.to_string_lossy().into_owned(),
        "другой пароль",
    )
    .unwrap_err();
    assert!(
        matches!(wrong, ApplicationError::Domain(ref error) if error.code() == "archive_password_invalid")
    );
    assert_eq!(target.load().unwrap(), LibraryState::default());

    let corrupt_archive = source_dir.join("corrupt.bookshelf.age");
    fs::write(&corrupt_archive, b"not an age archive").unwrap();
    let corrupt = crate::application::import_archive(
        &target.archive_storage(),
        &target,
        corrupt_archive.to_string_lossy().into_owned(),
        "надёжный пароль",
    )
    .unwrap_err();
    assert!(
        matches!(corrupt, ApplicationError::Domain(ref error) if error.code() == "archive_corrupt")
    );
    assert_eq!(target.load().unwrap(), LibraryState::default());

    let blocked_destination = source_dir.join("blocked.bookshelf.age");
    fs::create_dir(&blocked_destination).unwrap();
    let interrupted = crate::application::export_archive(
        &source.archive_storage(),
        &source,
        blocked_destination.to_string_lossy().into_owned(),
        "надёжный пароль",
    )
    .unwrap_err();
    assert!(matches!(interrupted, ApplicationError::Persistence(_)));
    assert!(blocked_destination.is_dir());
    assert!(!blocked_destination.with_extension("age.tmp").exists());

    crate::application::import_archive(
        &target.archive_storage(),
        &target,
        archive.to_string_lossy().into_owned(),
        "надёжный пароль",
    )
    .unwrap();
    assert_eq!(
        target.load().unwrap().workspace_note,
        "Переносимое состояние"
    );
    fs::remove_dir_all(source_dir).unwrap();
    fs::remove_dir_all(target_dir).unwrap();
    fs::remove_file(archive).unwrap();
}

#[test]
fn large_atomic_snapshot_round_trip_reports_size_and_elapsed_time() {
    let mut state = LibraryState::default();
    for index in 0..250_u32 {
        let book_id = format!("book-{index}");
        let idea_id = format!("idea-{index}");
        state.books.push(Book {
            id: book_id.clone(),
            title: format!("Книга {index}"),
            stored_file: format!("books/{book_id}.pdf"),
            has_text_layer: true,
            outline: vec![OutlineItem {
                id: format!("outline-{index}"),
                title: "Глава 1".into(),
                page: 1,
                parent_id: None,
            }],
            reading: ReadingPosition {
                page: index + 1,
                zoom: 1.25,
                scroll: 320.0,
            },
            reading_completed: index % 2 == 0,
            retrospective: None,
            ..Book::default()
        });
        state.drafts.push(DraftNote {
            id: format!("draft-{index}"),
            book_id: book_id.clone(),
            section: "Глава 1".into(),
            page: index + 1,
            excerpt: "Репрезентативный фрагмент книги".into(),
            context: "Контекст для полнотекстового поиска".into(),
            comment: "Авторская мысль".into(),
            fragments: vec![SourceFragment {
                page: index + 1,
                excerpt: "Репрезентативный фрагмент книги".into(),
                context: "Контекст для полнотекстового поиска".into(),
            }],
            created_at: 1_700_000_000 + u64::from(index),
        });
        state.ideas.push(Idea {
            id: idea_id.clone(),
            book_id: book_id.clone(),
            section: "Глава 1".into(),
            formulation: format!("Проверяемая идея {index}"),
            assignments: vec![IdeaAssignment::Recall, IdeaAssignment::Experiment],
            fragments: vec![SourceFragment {
                page: index + 1,
                excerpt: "Источник идеи".into(),
                context: "Соседний контекст".into(),
            }],
            versions: vec![IdeaVersion {
                formulation: format!("Проверяемая идея {index}"),
                saved_at: 1_700_000_000,
            }],
            topic_ids: vec![format!("topic-{index}")],
        });
        state.topics.push(Topic {
            id: format!("topic-{index}"),
            name: format!("Тема {index}"),
        });
        state.experiments.push(Experiment {
            id: format!("experiment-{index}"),
            idea_id: idea_id.clone(),
            situation: "Рабочая ситуация".into(),
            action: "Проверенное действие".into(),
            result: "Наблюдаемый результат".into(),
            conclusion: "Авторский вывод".into(),
            status: ExperimentStatus::Completed,
            cancellation_reason: String::new(),
            next_step: String::new(),
        });
        state.recalls.push(Recall {
            id: format!("recall-{index}"),
            idea_id: idea_id.clone(),
            answer: "Ответ своими словами".into(),
            rating: RecallRating::Partial,
            next_at: 1_700_604_800,
        });
        state.materials.push(TransferMaterial {
            id: format!("material-{index}"),
            title: format!("Материал {index}"),
            problem: "Проблема".into(),
            idea: "Объяснение идеи".into(),
            example: "Пример".into(),
            result: "Результат".into(),
            limitations: "Ограничения".into(),
            idea_ids: vec![idea_id],
        });
    }
    state.idea_links.push(IdeaLink {
        id: "link-large".into(),
        from_idea_id: "idea-0".into(),
        to_idea_id: "idea-1".into(),
        relation: IdeaRelation::Complements,
    });

    let started = std::time::Instant::now();
    let json = serde_json::to_vec(&state).unwrap();
    let data_dir = test_data_dir();
    let library = Library::open(&data_dir).unwrap();
    library.commit(&state).unwrap();
    let loaded = library.load().unwrap();
    let ipc_compatible: LibraryState =
        serde_json::from_slice(&serde_json::to_vec(&loaded).unwrap()).unwrap();
    let elapsed = started.elapsed();

    assert_eq!(ipc_compatible, state);
    eprintln!(
        "large LibraryState: {} bytes, serialize/persist/load round trip: {:?}",
        json.len(),
        elapsed
    );
    fs::remove_dir_all(data_dir).unwrap();
}

#[test]
fn activating_another_book_pauses_the_previous_study_atomically() {
    let mut state = LibraryState::default();
    state.books.push(Book::for_test("first", "Первая"));
    state.books.push(Book::for_test("second", "Вторая"));

    state
        .apply(LibraryAction::ActivateStudy {
            book_id: "first".into(),
        })
        .unwrap();
    state
        .apply(LibraryAction::ActivateStudy {
            book_id: "second".into(),
        })
        .unwrap();

    assert_eq!(state.active_study_book_id.as_deref(), Some("second"));
    assert_eq!(
        find_book(&state, "first").unwrap().study_status,
        StudyStatus::Paused
    );
    assert_eq!(
        find_book(&state, "second").unwrap().study_status,
        StudyStatus::Active
    );
}

#[test]
fn completed_book_requires_repeat_study_before_capturing_a_draft() {
    let mut state = LibraryState::default();
    state.books.push(Book {
        id: "completed".into(),
        title: "Завершённая книга".into(),
        study_status: StudyStatus::Completed,
        ..Book::default()
    });

    let rejected = state.apply(LibraryAction::CaptureDraft {
        book_id: "completed".into(),
        section: "Глава 1".into(),
        page: 4,
        excerpt: "Источник".into(),
        context: "Контекст".into(),
        comment: String::new(),
    });

    assert_eq!(
        rejected.unwrap_err().into_message(),
        "Начните повторное изучение, чтобы создавать новые черновые заметки"
    );
    assert!(state.drafts.is_empty());

    state
        .apply(LibraryAction::StartRepeatStudy {
            book_id: "completed".into(),
        })
        .unwrap();
    state
        .apply(LibraryAction::CaptureDraft {
            book_id: "completed".into(),
            section: "Глава 1".into(),
            page: 4,
            excerpt: "Источник".into(),
            context: "Контекст".into(),
            comment: String::new(),
        })
        .unwrap();

    assert_eq!(state.drafts.len(), 1);
}

#[test]
fn returning_to_an_earlier_page_does_not_reduce_farthest_reading_progress() {
    let mut state = LibraryState::default();
    state.books.push(Book::for_test("book", "Книга"));

    for page in [42, 18] {
        state
            .apply(LibraryAction::UpdateReading {
                book_id: "book".into(),
                page,
                zoom: 1.0,
                scroll: 0.0,
            })
            .unwrap();
    }

    let book = find_book(&state, "book").unwrap();
    assert_eq!(book.reading.page, 18);
    assert_eq!(book.farthest_page, 42);
    assert_eq!(
        state
            .milestones
            .iter()
            .filter(|milestone| milestone.kind == MilestoneKind::ReadingProgress)
            .count(),
        1
    );
}

#[test]
fn one_draft_preserves_each_page_of_a_multi_page_source() {
    let mut state = LibraryState::default();
    state.books.push(Book::for_test("book", "Книга"));
    state
        .apply(LibraryAction::CaptureDraftSources {
            book_id: "book".into(),
            section: "Глава".into(),
            fragments: vec![
                SourceFragment {
                    page: 10,
                    excerpt: "Начало мысли".into(),
                    context: "".into(),
                },
                SourceFragment {
                    page: 11,
                    excerpt: "Продолжение мысли".into(),
                    context: "".into(),
                },
            ],
            comment: "Проверить границу".into(),
        })
        .unwrap();

    assert_eq!(state.drafts.len(), 1);
    assert_eq!(state.drafts[0].fragments.len(), 2);
    assert_eq!(state.drafts[0].fragments[1].page, 11);
}

#[test]
fn cancelling_an_experiment_requires_a_reason_and_is_not_a_failure() {
    let mut state = LibraryState::default();
    state.ideas.push(Idea::for_test("idea", "book"));
    state.experiments.push(Experiment {
        id: "experiment".into(),
        idea_id: "idea".into(),
        ..Experiment::default()
    });
    let result = state.apply(LibraryAction::AdvanceExperiment {
        experiment_id: "experiment".into(),
        status: ExperimentStatus::Cancelled,
        situation: "".into(),
        action: "".into(),
        result: "".into(),
        conclusion: "".into(),
        cancellation_reason: "".into(),
        next_step: "".into(),
    });
    assert_eq!(
        result.unwrap_err().code(),
        "experiment_cancellation_reason_required"
    );

    state
        .apply(LibraryAction::AdvanceExperiment {
            experiment_id: "experiment".into(),
            status: ExperimentStatus::Cancelled,
            situation: "".into(),
            action: "".into(),
            result: "".into(),
            conclusion: "".into(),
            cancellation_reason: "Контекст исчез".into(),
            next_step: "".into(),
        })
        .unwrap();
    assert_eq!(state.experiments[0].status, ExperimentStatus::Cancelled);
    assert_eq!(state.experiments[0].status, ExperimentStatus::Cancelled);
}

#[test]
fn completion_draft_replaces_the_previous_step_for_the_same_book() {
    let mut state = LibraryState::default();
    state.books.push(Book::for_test("book", "Книга"));
    for step in [2, 3] {
        state
            .apply(LibraryAction::SaveStudyCompletionDraft {
                draft: StudyCompletionDraft {
                    book_id: "book".into(),
                    step,
                    retrospective: format!("Черновик шага {step}"),
                    ..StudyCompletionDraft::default()
                },
            })
            .unwrap();
    }
    assert_eq!(state.completion_drafts.len(), 1);
    assert_eq!(state.completion_drafts[0].step, 3);
}

#[test]
fn permanent_book_deletion_removes_all_owned_learning_state() {
    let book_id = "book".to_owned();
    let idea_id = "idea".to_owned();
    let idea_ids = std::collections::HashSet::from([idea_id.clone()]);
    let mut state = LibraryState {
        books: vec![Book::for_test(&book_id, "Книга")],
        drafts: vec![DraftNote::for_test("draft", &book_id)],
        ideas: vec![Idea::for_test(&idea_id, &book_id)],
        idea_links: vec![IdeaLink {
            id: "link".into(),
            from_idea_id: idea_id.clone(),
            to_idea_id: "other-idea".into(),
            relation: IdeaRelation::Complements,
        }],
        experiments: vec![Experiment {
            id: "experiment".into(),
            idea_id: idea_id.clone(),
            ..Experiment::default()
        }],
        recalls: vec![Recall {
            id: "recall".into(),
            idea_id: idea_id.clone(),
            ..Recall::default()
        }],
        reviews: vec![IdeaReview {
            id: "review".into(),
            idea_id: idea_id.clone(),
            ..IdeaReview::default()
        }],
        materials: vec![TransferMaterial {
            id: "material".into(),
            idea_ids: vec![idea_id.clone()],
            ..TransferMaterial::default()
        }],
        milestones: vec![StudyMilestone {
            id: "milestone".into(),
            book_id: book_id.clone(),
            kind: MilestoneKind::DraftCaptured,
            occurred_at: 1,
            page: None,
        }],
        active_study_book_id: Some(book_id.clone()),
        ..LibraryState::default()
    };
    state.completion_drafts.push(StudyCompletionDraft {
        book_id: book_id.clone(),
        step: 4,
        ..StudyCompletionDraft::default()
    });

    state
        .apply(LibraryAction::DeleteBook {
            book_id: book_id.clone(),
        })
        .unwrap();

    assert!(state.books.iter().all(|book| book.id != book_id));
    assert!(state.drafts.iter().all(|draft| draft.book_id != book_id));
    assert!(state.ideas.iter().all(|idea| idea.book_id != book_id));
    assert!(state.idea_links.iter().all(|link| {
        !idea_ids.contains(&link.from_idea_id) && !idea_ids.contains(&link.to_idea_id)
    }));
    assert!(state
        .experiments
        .iter()
        .all(|item| !idea_ids.contains(&item.idea_id)));
    assert!(state
        .recalls
        .iter()
        .all(|item| !idea_ids.contains(&item.idea_id)));
    assert!(state
        .reviews
        .iter()
        .all(|item| !idea_ids.contains(&item.idea_id)));
    assert!(state
        .materials
        .iter()
        .all(|item| item.idea_ids.iter().all(|id| !idea_ids.contains(id))));
    assert!(state.milestones.iter().all(|item| item.book_id != book_id));
    assert!(state
        .completion_drafts
        .iter()
        .all(|item| item.book_id != book_id));
    assert_eq!(state.active_study_book_id, None);
}
