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
        match self.read_state_file() {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(io::Error::other),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let recovery_file = self.state_file.with_extension("json.previous");
                match fs::read(recovery_file) {
                    Ok(bytes) => serde_json::from_slice(&bytes).map_err(io::Error::other),
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {
                        Ok(LibraryState::default())
                    }
                    Err(error) => Err(error),
                }
            }
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
        let recovery_file = self.state_file.with_extension("json.previous");
        let bytes = serde_json::to_vec_pretty(state).map_err(io::Error::other)?;

        remove_if_present(&temporary_file)?;
        let mut file = fs::File::create(&temporary_file)?;
        file.write_all(&bytes)?;
        file.sync_all()?;

        if self.state_file.exists() {
            remove_if_present(&recovery_file)?;
            fs::rename(&self.state_file, &recovery_file)?;
        }

        if let Err(error) = fs::rename(&temporary_file, &self.state_file) {
            if recovery_file.exists() {
                let _ = fs::rename(&recovery_file, &self.state_file);
            }
            return Err(error);
        }

        remove_if_present(&recovery_file)
    }

    fn read_state_file(&self) -> io::Result<Vec<u8>> {
        fs::read(&self.state_file)
    }
}

fn remove_if_present(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
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

    #[test]
    fn repeated_workspace_changes_replace_state_portably() {
        let data_dir = test_data_dir();
        let library = Library::open(&data_dir).expect("library should open");

        library
            .save_workspace_note("Первая пометка".into())
            .expect("first change should save");
        library
            .save_workspace_note("Актуальная пометка".into())
            .expect("replacement should save");

        assert_eq!(
            library
                .load()
                .expect("replacement should load")
                .workspace_note,
            "Актуальная пометка"
        );
        assert!(!data_dir.join("library.json.previous").exists());

        fs::remove_dir_all(data_dir).expect("test data should be removable");
    }

    #[test]
    fn interrupted_replacement_recovers_the_previous_state() {
        let data_dir = test_data_dir();
        let library = Library::open(&data_dir).expect("library should open");
        library
            .save_workspace_note("Сохранённая пометка".into())
            .expect("change should save");
        fs::rename(
            data_dir.join("library.json"),
            data_dir.join("library.json.previous"),
        )
        .expect("interrupted swap should be simulated");

        assert_eq!(
            library
                .load()
                .expect("previous state should recover")
                .workspace_note,
            "Сохранённая пометка"
        );

        fs::remove_dir_all(data_dir).expect("test data should be removable");
    }
}
