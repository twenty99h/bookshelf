use super::*;

impl Library {
    pub fn export_material_markdown(
        &self,
        material_id: &str,
        destination: impl AsRef<Path>,
    ) -> Result<(), LibraryError> {
        let state = self.load()?;
        let material = state
            .materials
            .iter()
            .find(|item| item.id == material_id)
            .ok_or_else(|| DomainError::new("material_not_found", "Материал не найден"))?;
        let sources = material
            .idea_ids
            .iter()
            .filter_map(|id| state.ideas.iter().find(|idea| &idea.id == id))
            .map(|idea| {
                let book = state
                    .books
                    .iter()
                    .find(|book| book.id == idea.book_id)
                    .map(|book| book.title.as_str())
                    .unwrap_or("Книга");
                format!("- {book}, {} — {}", idea.section, idea.formulation)
            })
            .collect::<Vec<_>>()
            .join("\n");
        let markdown = format!("# {}\n\n## Проблема\n\n{}\n\n## Идея\n\n{}\n\n## Пример применения\n\n{}\n\n## Результат\n\n{}\n\n## Ограничения\n\n{}\n\n## Источники\n\n{}\n", material.title, material.problem, material.idea, material.example, material.result, material.limitations, sources);
        atomic_write(destination.as_ref(), markdown.as_bytes())?;
        Ok(())
    }

    pub fn export_draft_markdown(
        &self,
        draft_id: &str,
        destination: impl AsRef<Path>,
    ) -> Result<LibraryState, LibraryError> {
        let state = self.load()?;
        let draft = state
            .drafts
            .iter()
            .find(|item| item.id == draft_id)
            .ok_or_else(|| DomainError::new("draft_not_found", "Черновая заметка не найдена"))?;
        let book = state
            .books
            .iter()
            .find(|book| book.id == draft.book_id)
            .map(|book| book.title.as_str())
            .unwrap_or("Книга");
        let markdown = format!(
            "# Черновая заметка\n\nИсточник: {book}, {}, стр. {}\n\n> {}\n\n{}\n",
            draft.section, draft.page, draft.excerpt, draft.comment
        );
        atomic_write(destination.as_ref(), markdown.as_bytes())?;
        self.apply(LibraryAction::DiscardDraft {
            draft_id: draft_id.into(),
        })
    }
}
