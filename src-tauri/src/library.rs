use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryState {
    pub books: Vec<BookSummary>,
    pub workspace_note: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookSummary {
    pub id: String,
    pub title: String,
}

pub struct Library {
    state_file: PathBuf,
}

impl Library {
    pub fn open(data_dir: impl AsRef<Path>) -> io::Result<Self> {
        fs::create_dir_all(data_dir.as_ref())?;
        Ok(Self {
            state_file: data_dir.as_ref().join("library.json"),
        })
    }

    pub fn load(&self) -> io::Result<LibraryState> {
        match fs::read(&self.state_file) {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(io::Error::other),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(LibraryState::default()),
            Err(error) => Err(error),
        }
    }

    pub fn save_workspace_note(&self, note: String) -> io::Result<LibraryState> {
        let mut state = self.load()?;
        state.workspace_note = note;
        self.replace_state(&state)?;
        Ok(state)
    }

    fn replace_state(&self, state: &LibraryState) -> io::Result<()> {
        let temporary_file = self.state_file.with_extension("json.tmp");
        let bytes = serde_json::to_vec_pretty(state).map_err(io::Error::other)?;

        let mut file = fs::File::create(&temporary_file)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        fs::rename(temporary_file, &self.state_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_data_dir() -> PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after the epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("bookshelf-library-test-{id}"))
    }

    #[test]
    fn clean_launch_opens_an_empty_personal_library() {
        let data_dir = test_data_dir();
        let library = Library::open(&data_dir).expect("library should open");

        assert_eq!(
            library.load().expect("state should load"),
            LibraryState::default()
        );

        fs::remove_dir_all(data_dir).expect("test data should be removable");
    }

    #[test]
    fn workspace_change_survives_a_desktop_restart() {
        let data_dir = test_data_dir();
        let first_launch = Library::open(&data_dir).expect("first launch should open");
        first_launch
            .save_workspace_note("Продолжить с главы 2".into())
            .expect("change should save");
        drop(first_launch);

        let restarted = Library::open(&data_dir).expect("restarted app should open");
        assert_eq!(
            restarted
                .load()
                .expect("saved state should load")
                .workspace_note,
            "Продолжить с главы 2"
        );

        fs::remove_dir_all(data_dir).expect("test data should be removable");
    }
}
