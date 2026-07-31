use super::*;

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
