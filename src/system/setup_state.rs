//! The state of each post-install task. The `crownos-setup` runner writes one
//! file for each task, `<SETUP_STATE_DIR>/<task id>.ron`. The runner writes
//! each file atomically: it writes a temporary file in the same directory and
//! then renames it. Thus a reader never sees a half-written file.

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

pub const SETUP_STATE_DIR: &str = "/var/lib/crownos/setup";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SetupTask {
    Browser,
    DeveloperTools,
    Playwright,
}

#[derive(Debug, thiserror::Error)]
#[error("unknown setup task `{0}`")]
pub struct UnknownSetupTask(pub String);

impl SetupTask {
    pub const ALL: [SetupTask; 3] = [Self::Browser, Self::DeveloperTools, Self::Playwright];

    pub fn id(self) -> &'static str {
        match self {
            Self::Browser => "browser",
            Self::DeveloperTools => "developer-tools",
            Self::Playwright => "playwright",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Browser => "Web browser",
            Self::DeveloperTools => "Developer tools",
            Self::Playwright => "Playwright",
        }
    }
}

impl FromStr for SetupTask {
    type Err = UnknownSetupTask;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|task| task.id() == id)
            .ok_or_else(|| UnknownSetupTask(id.to_owned()))
    }
}

impl fmt::Display for SetupTask {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.id())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Done,
    Failed { message: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskState {
    pub status: TaskStatus,
    #[serde(default)]
    pub generation: u64,
    #[serde(default)]
    pub started_at_unix: Option<u64>,
    #[serde(default)]
    pub finished_at_unix: Option<u64>,
}

#[derive(Debug, thiserror::Error)]
pub enum SetupStateError {
    #[error("could not read {}: {source}", path.display())]
    Read { path: PathBuf, source: io::Error },
    #[error("could not parse {}: {source}", path.display())]
    Parse {
        path: PathBuf,
        source: Box<ron::error::SpannedError>,
    },
    #[error("could not render a task state: {0}")]
    Render(#[from] ron::Error),
}

impl TaskState {
    pub fn path(dir: &Path, task: SetupTask) -> PathBuf {
        dir.join(format!("{}.ron", task.id()))
    }

    /// The state of `task`, or `None` when the runner did not write it yet.
    pub fn load(dir: &Path, task: SetupTask) -> Result<Option<Self>, SetupStateError> {
        let path = Self::path(dir, task);
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(source) => return Err(SetupStateError::Read { path, source }),
        };
        ron::from_str(&text)
            .map(Some)
            .map_err(|source| SetupStateError::Parse {
                path,
                source: Box::new(source),
            })
    }

    /// The tasks that have a state file, in [`SetupTask::ALL`] order.
    pub fn load_all(dir: &Path) -> Result<Vec<(SetupTask, Self)>, SetupStateError> {
        let mut states = Vec::new();
        for task in SetupTask::ALL {
            if let Some(state) = Self::load(dir, task)? {
                states.push((task, state));
            }
        }
        Ok(states)
    }

    /// The task states of the installed system.
    pub fn load_all_installed() -> Result<Vec<(SetupTask, Self)>, SetupStateError> {
        Self::load_all(Path::new(SETUP_STATE_DIR))
    }

    pub fn is_settled(&self) -> bool {
        matches!(self.status, TaskStatus::Done | TaskStatus::Failed { .. })
    }

    pub fn render(&self) -> Result<String, SetupStateError> {
        Ok(ron::ser::to_string_pretty(
            self,
            ron::ser::PrettyConfig::default(),
        )?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_temp_dir(test_name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("crownos-config-{test_name}-{}", std::process::id()));
        if let Err(error) = std::fs::remove_dir_all(&dir) {
            assert_eq!(error.kind(), io::ErrorKind::NotFound, "{error}");
        }
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    fn failed_state() -> TaskState {
        TaskState {
            status: TaskStatus::Failed {
                message: "pacman exited with status 1".to_owned(),
            },
            generation: 3,
            started_at_unix: Some(1_700_000_000),
            finished_at_unix: Some(1_700_000_042),
        }
    }

    #[test]
    fn render_round_trips() {
        let state = failed_state();
        let text = state.render().expect("serializable");
        assert_eq!(ron::from_str::<TaskState>(&text).expect("valid"), state);
        assert!(state.is_settled());
    }

    #[test]
    fn a_status_alone_parses_with_default_fields() {
        let state: TaskState = ron::from_str("(status: Pending)").expect("valid");
        assert_eq!(state.status, TaskStatus::Pending);
        assert_eq!(state.generation, 0);
        assert_eq!(state.started_at_unix, None);
        assert_eq!(state.finished_at_unix, None);
        assert!(!state.is_settled());
    }

    #[test]
    fn task_ids_round_trip() {
        for task in SetupTask::ALL {
            assert_eq!(task.id().parse::<SetupTask>().expect("known"), task);
            assert_eq!(task.to_string(), task.id());
        }
        assert!("edge".parse::<SetupTask>().is_err());
    }

    #[test]
    fn all_lists_every_task() {
        let mut seen = [false; SetupTask::ALL.len()];
        for task in SetupTask::ALL {
            let index = match task {
                SetupTask::Browser => 0,
                SetupTask::DeveloperTools => 1,
                SetupTask::Playwright => 2,
            };
            seen[index] = true;
        }
        assert!(seen.iter().all(|&listed| listed));
    }

    #[test]
    fn a_missing_directory_has_no_states() {
        let missing = Path::new("/nonexistent/crownos/setup");
        assert!(TaskState::load_all(missing).expect("readable").is_empty());
    }

    #[test]
    fn load_all_finds_a_written_state() {
        let dir = fresh_temp_dir("load_all_finds_a_written_state");
        let state = TaskState {
            status: TaskStatus::Running,
            generation: 1,
            started_at_unix: Some(1_700_000_000),
            finished_at_unix: None,
        };
        assert!(!state.is_settled());
        let written = std::fs::write(
            TaskState::path(&dir, SetupTask::Playwright),
            state.render().expect("serializable"),
        );
        let loaded = TaskState::load_all(&dir);
        std::fs::remove_dir_all(&dir).expect("cleanup");
        written.expect("write");
        assert_eq!(loaded.expect("readable"), [(SetupTask::Playwright, state)]);
    }

    #[test]
    fn load_all_fails_on_a_malformed_state() {
        let dir = fresh_temp_dir("load_all_fails_on_a_malformed_state");
        let written = std::fs::write(TaskState::path(&dir, SetupTask::Browser), "(status: ");
        let loaded = TaskState::load_all(&dir);
        std::fs::remove_dir_all(&dir).expect("cleanup");
        written.expect("write");
        match loaded {
            Err(SetupStateError::Parse { path, .. }) => assert!(path.ends_with("browser.ron")),
            other => panic!("expected a parse error, got {other:?}"),
        }
    }
}
