//! In-memory index over the per-log history files.
//!
//! Built lazily and refreshed by mtime: each call to `refresh` stats the
//! history directory and re-parses only files that are new or changed, so the
//! app's own writes and external backfills are both picked up without any
//! invalidation plumbing.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use hashbrown::{HashMap, HashSet};

use baras_types::history::{BossNode, PullFilter, PullHistoryOverview, PullRow};

use super::{DUMMY_GROUP, LogHistory, history_dir};
use crate::encounter::PhaseType;
use crate::encounter::summary::EncounterSummary;

struct IndexedLog {
    modified: SystemTime,
    /// Where the log should be; existence is checked at query time
    log_path: PathBuf,
    rows: Vec<PullRow>,
}

#[derive(Default)]
pub struct PullIndex {
    logs: HashMap<String, IndexedLog>,
}

impl PullIndex {
    /// Sync with the history directory. `log_dir` is the fallback location
    /// for logs recorded before paths were stored.
    pub fn refresh(&mut self, log_dir: &Path) -> std::io::Result<()> {
        let mut seen = HashSet::new();
        for entry in std::fs::read_dir(history_dir()?)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_none_or(|e| e != "json") {
                continue;
            }
            let Some(name) = path.file_name().and_then(|f| f.to_str()).map(str::to_string) else {
                continue;
            };
            let modified = entry.metadata()?.modified()?;
            seen.insert(name.clone());
            if self.logs.get(&name).is_some_and(|l| l.modified == modified) {
                continue;
            }
            match std::fs::read(&path)
                .map_err(|e| e.to_string())
                .and_then(|b| serde_json::from_slice::<LogHistory>(&b).map_err(|e| e.to_string()))
            {
                Ok(history) => {
                    let log_path = history.log_path(log_dir);
                    let rows = history.pulls.iter().map(|s| row_from(&history, s)).collect();
                    self.logs.insert(name, IndexedLog { modified, log_path, rows });
                }
                Err(e) => tracing::warn!(path = %path.display(), error = %e, "Skipping unreadable history file"),
            }
        }
        self.logs.retain(|name, _| seen.contains(name));
        Ok(())
    }

    /// Filter option lists (unfiltered) plus per-boss pull/kill counts under `filter`.
    pub fn overview(&self, filter: &PullFilter) -> PullHistoryOverview {
        let characters = distinct(self.rows().map(|r| r.character.as_str()));
        let disciplines = distinct(self.rows().filter_map(|r| r.discipline.as_deref()));

        let mut counts: HashMap<(&str, &str), (u32, u32)> = HashMap::new();
        for r in self.rows().filter(|r| matches(r, filter)) {
            let e = counts.entry((&r.operation, &r.boss)).or_default();
            e.0 += 1;
            e.1 += u32::from(r.success);
        }
        let mut bosses: Vec<BossNode> = counts
            .into_iter()
            .map(|((operation, boss), (pulls, kills))| BossNode {
                operation: operation.to_string(),
                boss: boss.to_string(),
                pulls,
                kills,
            })
            .collect();
        bosses.sort_by(|a, b| {
            (a.operation == DUMMY_GROUP, &a.operation, &a.boss)
                .cmp(&(b.operation == DUMMY_GROUP, &b.operation, &b.boss))
        });
        PullHistoryOverview { characters, disciplines, bosses }
    }

    /// Every pull of one boss under `filter`, newest first, with `path` set
    /// when the log exists.
    pub fn pulls(&self, operation: &str, boss: &str, filter: &PullFilter) -> Vec<PullRow> {
        let mut out = Vec::new();
        for log in self.logs.values() {
            let mut exists = None;
            for r in log
                .rows
                .iter()
                .filter(|r| r.operation == operation && r.boss == boss && matches(r, filter))
            {
                let exists = *exists.get_or_insert_with(|| log.log_path.is_file());
                let mut row = r.clone();
                row.path = exists.then(|| log.log_path.to_string_lossy().into_owned());
                out.push(row);
            }
        }
        out.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        out
    }

    fn rows(&self) -> impl Iterator<Item = &PullRow> {
        self.logs.values().flat_map(|l| l.rows.iter())
    }
}

fn matches(r: &PullRow, f: &PullFilter) -> bool {
    (f.character.is_empty() || r.character == f.character)
        && (f.role.is_empty() || r.role.as_deref() == Some(f.role.as_str()))
        && (f.discipline.is_empty() || r.discipline.as_deref() == Some(f.discipline.as_str()))
        && (f.tier.is_empty() || r.difficulty.contains(f.tier.as_str()))
        && (!f.operations_only || r.is_operation)
        && (!f.kills_only || r.success)
        && in_date_range(&r.timestamp, f)
}

/// ISO timestamps compare lexicographically, so day bounds need no parsing
fn in_date_range(timestamp: &str, f: &PullFilter) -> bool {
    let day = timestamp.get(..10).unwrap_or(timestamp);
    (f.date_from.is_empty() || day >= f.date_from.as_str())
        && (f.date_to.is_empty() || day <= f.date_to.as_str())
}

fn distinct<'a>(it: impl Iterator<Item = &'a str>) -> Vec<String> {
    let mut v: Vec<String> = it.collect::<HashSet<_>>().into_iter().map(str::to_string).collect();
    v.sort();
    v
}

fn row_from(log: &LogHistory, s: &EncounterSummary) -> PullRow {
    let character = log.character_name.clone().unwrap_or_else(|| "Unknown".into());
    let me = s.player_metrics.iter().find(|m| m.name == character);
    let (operation, boss) = if s.encounter_type == PhaseType::DummyParse {
        (DUMMY_GROUP.to_string(), DUMMY_GROUP.to_string())
    } else {
        (
            s.area_name.clone(),
            s.boss_name.clone().unwrap_or_else(|| s.display_name.clone()),
        )
    };
    PullRow {
        filename: log.filename.clone(),
        path: None,
        encounter_id: s.encounter_id,
        timestamp: s
            .start_time
            .clone()
            .or_else(|| log.started_at.map(|t| t.format("%Y-%m-%dT%H:%M:%S").to_string()))
            .unwrap_or_default(),
        operation,
        boss,
        difficulty: s.difficulty.clone().unwrap_or_else(|| "-".into()),
        success: s.success,
        duration_seconds: s.duration_seconds,
        is_operation: s.encounter_type == PhaseType::Raid,
        discipline: me.and_then(|m| m.discipline_name.clone()),
        role: me.and_then(|m| m.discipline.map(|d| format!("{:?}", d.role()))),
        role_icon: me.and_then(|m| m.role_icon.clone()),
        discipline_icon: me.and_then(|m| m.discipline.map(|d| d.icon_name().to_string())),
        dps: me.map(|m| m.dps),
        hps: me.map(|m| m.hps),
        character,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Dev benchmark against the real history store (skipped in CI).
    /// `cargo test -p baras-core --lib history::index -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn refresh_timing_on_real_store() {
        let mut index = PullIndex::default();
        let t = std::time::Instant::now();
        index.refresh(Path::new(".")).unwrap();
        let cold = t.elapsed();
        let t = std::time::Instant::now();
        index.refresh(Path::new(".")).unwrap();
        let warm = t.elapsed();
        let t = std::time::Instant::now();
        let filter = PullFilter::default();
        let overview = index.overview(&filter);
        let pulls = index.pulls(&overview.bosses[0].operation, &overview.bosses[0].boss, &filter);
        let query = t.elapsed();
        println!(
            "logs={} bosses={} first_boss_pulls={} cold={cold:?} warm={warm:?} query={query:?}",
            index.logs.len(),
            overview.bosses.len(),
            pulls.len()
        );
    }
}
