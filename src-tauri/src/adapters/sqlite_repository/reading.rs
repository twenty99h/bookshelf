use super::*;
use sha2::{Digest, Sha256};

impl Library {
    pub(super) fn store_pdf(
        &self,
        source: impl AsRef<Path>,
        title: String,
        id: String,
    ) -> Result<Book, LibraryError> {
        let bytes = fs::read(source.as_ref())?;
        let document = lopdf::Document::load_mem(&bytes).map_err(|_| {
            DomainError::new("pdf_invalid", "Выбранный файл не является корректным PDF")
        })?;
        let has_text_layer = document.get_pages().values().any(|page_id| {
            document
                .get_page_content_with_limit(*page_id, 8 * 1024 * 1024)
                .ok()
                .and_then(|content| lopdf::content::Content::decode(&content).ok())
                .is_some_and(|content| {
                    content.operations.iter().any(|operation| {
                        matches!(operation.operator.as_str(), "Tj" | "TJ" | "'" | "\"")
                    })
                })
        });
        let page_count = document.get_pages().len() as u32;
        let content_hash = format!("{:x}", Sha256::digest(&bytes));
        let title = if title.trim().is_empty() {
            source
                .as_ref()
                .file_stem()
                .and_then(|item| item.to_str())
                .unwrap_or("Книга")
                .to_owned()
        } else {
            title
        };
        let relative = format!("books/{id}.pdf");
        let target = self.data_dir.join(&relative);
        let temporary = target.with_extension("pdf.tmp");
        fs::write(&temporary, bytes)?;
        fs::rename(&temporary, &target)?;
        Ok(Book {
            id,
            title,
            stored_file: relative,
            has_text_layer,
            page_count,
            content_hash,
            farthest_page: 1,
            ..Book::default()
        })
    }
}
