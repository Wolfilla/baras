//! Cross-session pull history.
//!
//! Boss pulls and dummy parses are persisted as one JSON file per combat log
//! under `~/.config/baras/history/`, so the Historical tab can compare pulls
//! across nights without re-parsing logs. Each file is rewritten in full from
//! the session's `EncounterHistory`, which makes re-parsing a log idempotent.

mod index;

use std::path::{Path, PathBuf};

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::context::parse_log_filename;
use crate::encounter::PhaseType;
use crate::encounter::summary::EncounterSummary;

pub use index::PullIndex;

/// Tree group for training dummy parses (they have no boss or operation)
pub const DUMMY_GROUP: &str = "Training Dummy";

/// Recorded pulls from a single combat log file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogHistory {
    /// Combat log filename (e.g. `combat_2026-09-29_02_20_23_294905.txt`)
    pub filename: String,
    /// Full path the log was parsed from. Cleared on load if the file is gone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
    /// Local player for this log (None if never initialized)
    pub character_name: Option<String>,
    /// Log start time parsed from the filename
    pub started_at: Option<NaiveDateTime>,
    pub pulls: Vec<EncounterSummary>,
}

/// `~/.config/baras/history/` (created on demand)
pub fn history_dir() -> std::io::Result<PathBuf> {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("baras")
        .join("history");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Boss pulls (any content type except PvP) and training dummy parses.
pub fn is_recordable(summary: &EncounterSummary) -> bool {
    match summary.encounter_type {
        PhaseType::DummyParse => true,
        PhaseType::PvP => false,
        _ => summary.boss_name.is_some(),
    }
}

/// Drop fields that only matter within the live session.
fn strip_bulk(summary: &EncounterSummary) -> EncounterSummary {
    let mut s = summary.clone();
    s.incoming_damage = Vec::new();
    s.area_entered_line = None;
    s.event_start_line = None;
    s.event_end_line = None;
    s
}

impl LogHistory {
    /// Build from a session's encounter history, keeping only recordable pulls.
    /// Returns None if `path` has no usable filename.
    pub fn from_summaries(
        path: &Path,
        character_name: Option<String>,
        summaries: &[EncounterSummary],
    ) -> Option<Self> {
        let filename = path.file_name()?.to_str()?;
        Some(Self {
            filename: filename.to_string(),
            path: Some(path.to_path_buf()),
            character_name,
            started_at: parse_log_filename(filename).map(|(_, dt)| dt),
            pulls: summaries.iter().filter(|s| is_recordable(s)).map(strip_bulk).collect(),
        })
    }

    /// Where the log should be: the stored path, or `log_dir/filename` for
    /// files recorded before paths were stored.
    pub fn log_path(&self, log_dir: &Path) -> PathBuf {
        self.path
            .clone()
            .unwrap_or_else(|| log_dir.join(&self.filename))
    }

    fn path(&self) -> std::io::Result<PathBuf> {
        let stem = self.filename.strip_suffix(".txt").unwrap_or(&self.filename);
        Ok(history_dir()?.join(format!("{stem}.json")))
    }

    /// Write atomically (tmp + rename). A log with no recordable pulls
    /// removes any stale file left from an earlier parse.
    pub fn save(&self) -> std::io::Result<()> {
        let path = self.path()?;
        if self.pulls.is_empty() {
            return match std::fs::remove_file(&path) {
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                other => other,
            };
        }
        let tmp = path.with_extension("json.tmp");
        let bytes = serde_json::to_vec(self).map_err(std::io::Error::other)?;
        std::fs::write(&tmp, bytes)?;
        std::fs::rename(&tmp, &path)
    }
}
