use super::*;

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
