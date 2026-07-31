use super::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct Book {
    pub id: String,
    pub title: String,
    pub stored_file: String,
    pub has_text_layer: bool,
    pub outline: Vec<OutlineItem>,
    pub reading: ReadingPosition,
    pub farthest_page: u32,
    pub page_count: u32,
    pub content_hash: String,
    pub reader: ReaderPreferences,
    pub study_status: StudyStatus,
    pub study_cycles: Vec<StudyCycle>,
    pub archived: bool,
    pub reading_completed: bool,
    pub retrospective: Option<Retrospective>,
}

impl Book {
    #[cfg(test)]
    pub(crate) fn for_test(id: &str, title: &str) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            has_text_layer: true,
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct OutlineItem {
    pub id: String,
    pub title: String,
    pub page: u32,
    pub parent_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ReadingPosition {
    pub page: u32,
    pub zoom: f32,
    pub scroll: f32,
}

impl Default for ReadingPosition {
    fn default() -> Self {
        Self {
            page: 1,
            zoom: 1.0,
            scroll: 0.0,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ReaderPreferences {
    pub document_mode: DocumentMode,
    pub invert_images: bool,
    pub sidebar_open: bool,
    pub sidebar_tab: ReaderSidebarTab,
    pub sidebar_width: u16,
}

impl Default for ReaderPreferences {
    fn default() -> Self {
        Self {
            document_mode: DocumentMode::MutedLight,
            invert_images: true,
            sidebar_open: false,
            sidebar_tab: ReaderSidebarTab::Note,
            sidebar_width: 400,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum DocumentMode {
    #[default]
    MutedLight,
    Original,
    DarkInverted,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum ReaderSidebarTab {
    #[default]
    Note,
    Outline,
    Search,
}

pub(super) fn validate_outline(outline: &[OutlineItem]) -> Result<(), DomainError> {
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
    Ok(())
}

pub(super) fn apply(
    state: &mut LibraryState,
    action: LibraryAction,
    timestamp: u64,
    make_id: &mut impl FnMut(&str) -> String,
) -> Result<(), DomainError> {
    match action {
        LibraryAction::UpdateReading {
            book_id,
            page,
            zoom,
            scroll,
        } => {
            let book = find_book_mut(state, &book_id)?;
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
                state.milestones.push(StudyMilestone {
                    id: make_id("milestone"),
                    book_id,
                    kind: MilestoneKind::ReadingProgress,
                    occurred_at: timestamp,
                    page: Some(page),
                });
            }
        }
        LibraryAction::SaveOutline { book_id, outline } => {
            validate_outline(&outline)?;
            find_book_mut(state, &book_id)?.outline = outline;
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
            find_book_mut(state, &book_id)?.reader = preferences;
        }
        _ => unreachable!("action dispatched to the wrong capability"),
    }
    Ok(())
}
