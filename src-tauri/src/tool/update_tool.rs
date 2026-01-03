#[cfg(desktop)]
use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::Update;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Updater(#[from] tauri_plugin_updater::Error),
    #[error("there is no pending update")]
    NoPendingUpdate,
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

#[derive(Clone, Serialize)]
#[serde(tag = "event", content = "data")]
pub enum DownloadEvent {
    #[serde(rename_all = "camelCase")]
    Started {
        content_length: Option<u64>,
    },
    #[serde(rename_all = "camelCase")]
    Progress {
        chunk_length: usize,
    },
    Finished,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMetadata {
    pub version: String,
    pub current_version: String,
}

pub struct PendingUpdate(pub Mutex<Option<Update>>);

pub fn check_update(app: &AppHandle) {
    let _ = app.emit("check_update", "");
}
