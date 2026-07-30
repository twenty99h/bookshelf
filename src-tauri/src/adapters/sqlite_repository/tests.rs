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
fn resolving_a_draft_requires_an_authored_idea_source_and_assignment() {
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
            successful: false,
        })
        .unwrap();
    assert!(!state.experiments[0].successful);
    assert!(state.experiments[0].completed);
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
        debt_decision: "".into(),
    });
    assert_eq!(result.unwrap_err().code(), "retrospective_required");
    assert_eq!(state.active_study_book_id.as_deref(), Some("book"));
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
fn completing_a_session_reports_debt_change_since_it_started() {
    let data_dir = test_data_dir();
    let library = Library::open(&data_dir).unwrap();
    let mut state = LibraryState::default();
    state.books.push(Book::for_test("book", "Книга"));
    library.replace_state(&state).unwrap();
    let planned = library
        .apply(LibraryAction::PlanSession {
            intention: "Продолжить чтение".into(),
            planned_at: 10,
        })
        .unwrap();
    let session_id = planned.sessions[0].id.clone();
    library
        .apply(LibraryAction::CaptureDraft {
            book_id: "book".into(),
            section: "Глава".into(),
            page: 1,
            excerpt: "Фрагмент".into(),
            context: "".into(),
            comment: "".into(),
        })
        .unwrap();
    library
        .apply(LibraryAction::StartSession {
            session_id: session_id.clone(),
        })
        .unwrap();
    library
        .apply(LibraryAction::CaptureDraft {
            book_id: "book".into(),
            section: "Глава".into(),
            page: 2,
            excerpt: "Второй фрагмент".into(),
            context: "".into(),
            comment: "".into(),
        })
        .unwrap();
    let completed = library
        .apply(LibraryAction::ResolveSession {
            session_id,
            status: SessionStatus::Completed,
            reason: "".into(),
        })
        .unwrap();
    assert_eq!(completed.last_debt_change, 1);
    fs::remove_dir_all(data_dir).unwrap();
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
fn pdf_import_corpus_covers_text_variants_and_rejects_scanned_input() {
    let data_dir = test_data_dir();
    let library = Library::open(&data_dir).unwrap();
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pdf");
    let text_layer = corpus.join("text-layer.pdf");
    let state = crate::application::import_pdf(
        &library.reading_storage(),
        &library,
        &SystemIdGenerator,
        text_layer.to_string_lossy().into_owned(),
        String::new(),
    )
    .unwrap();
    let imported = state.books.last().unwrap();
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
    let state = crate::application::import_pdf(
        &library.reading_storage(),
        &library,
        &SystemIdGenerator,
        complex.to_string_lossy().into_owned(),
        "Спецификация с оглавлением".into(),
    )
    .unwrap();
    assert!(state.books.last().unwrap().has_text_layer);

    let scanned = corpus.join("image-only.pdf");
    let error = crate::application::import_pdf(
        &library.reading_storage(),
        &library,
        &SystemIdGenerator,
        scanned.to_string_lossy().into_owned(),
        "Скан".into(),
    )
    .unwrap_err();
    assert!(
        matches!(error, ApplicationError::Domain(ref error) if error.code() == "pdf_text_layer_missing")
    );
    fs::remove_dir_all(data_dir).unwrap();
}

#[test]
fn unchanged_debt_produces_only_one_notification_after_the_configured_period() {
    let data_dir = test_data_dir();
    let library = Library::open(&data_dir).unwrap();
    let mut state = LibraryState::default();
    state.books.push(Book::for_test("book", "Книга"));
    state.drafts.push(DraftNote::for_test("draft", "book"));
    state.debt_reminder_days = 3;
    state.last_debt_changed_at = now() - 4 * 86_400;
    library.replace_state(&state).unwrap();
    assert_eq!(library.claim_debt_notification().unwrap(), Some(1));
    assert_eq!(library.claim_debt_notification().unwrap(), None);
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
            study_completed: false,
            retrospective: None,
        });
        state.drafts.push(DraftNote {
            id: format!("draft-{index}"),
            book_id: book_id.clone(),
            section: "Глава 1".into(),
            page: index + 1,
            excerpt: "Репрезентативный фрагмент книги".into(),
            context: "Контекст для полнотекстового поиска".into(),
            comment: "Авторская мысль".into(),
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
            successful: index % 2 == 0,
            completed: true,
        });
        state.recalls.push(Recall {
            id: format!("recall-{index}"),
            idea_id: idea_id.clone(),
            answer: "Ответ своими словами".into(),
            rating: RecallRating::Partial,
            next_at: 1_700_604_800,
        });
        state.sessions.push(StudySession {
            id: format!("session-{index}"),
            intention: "Продолжить изучение".into(),
            planned_at: 1_700_000_000,
            status: SessionStatus::Completed,
            resolution_reason: String::new(),
            debt_at_start: 2,
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
