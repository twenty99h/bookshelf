use super::*;

impl Library {
    pub fn search(&self, query: &str) -> io::Result<Vec<SearchResult>> {
        let terms = query
            .split_whitespace()
            .map(|term| format!("\"{}\"", term.replace('"', "\"\"")))
            .collect::<Vec<_>>()
            .join(" AND ");
        if terms.is_empty() {
            return Ok(vec![]);
        }
        let connection = Connection::open(&self.database_file).map_err(sqlite_io)?;
        let mut statement = connection.prepare("SELECT entity_id, kind, title, context FROM search_index WHERE search_index MATCH ?1 ORDER BY rank LIMIT 50").map_err(sqlite_io)?;
        let rows = statement
            .query_map([terms], |row| {
                let stored_kind: String = row.get(1)?;
                let kind = SearchResultKind::from_database(&stored_kind)
                    .ok_or(rusqlite::Error::InvalidQuery)?;
                Ok(SearchResult {
                    id: row.get(0)?,
                    kind,
                    title: row.get(2)?,
                    context: row.get(3)?,
                })
            })
            .map_err(sqlite_io)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(sqlite_io)
    }
}
