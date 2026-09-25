use crate::clock::now;
use crate::error::Result;
use crate::fill::FillReport;
use intl_ai_formats::json::write_atomic;
use serde::Serialize;
use std::path::Path;

/// `.intl-ai/report-<ts>.json` (plan 5.1.8): written iff there were failures
/// and the run was not a dry-run.
#[derive(Debug, Serialize)]
pub struct RunReport<'a> {
    pub version: u32,
    pub generated_at: String,
    pub command: &'a str,
    #[serde(flatten)]
    pub payload: &'a FillReport,
}

pub fn write_if_failures(cfg_dir: &Path, report: &FillReport) -> Result<Option<String>> {
    if report.dry_run || report.failures.is_empty() {
        return Ok(None);
    }
    let dir = cfg_dir.join(".intl-ai");
    let name = format!("report-{}.json", now().replace(':', "-"));
    let path = dir.join(&name);
    let body = serde_json::to_string_pretty(&RunReport {
        version: 1,
        generated_at: now(),
        command: "fill",
        payload: report,
    })
    .unwrap_or_default();
    write_atomic(&path, format!("{body}\n").as_bytes())?;
    gc_reports(&dir);
    Ok(Some(path.display().to_string()))
}

/// Report files accumulate; keep only the newest few (timestamped names
/// sort chronologically). Best-effort: a delete failure is not worth
/// failing a run that already wrote its report.
const KEEP_REPORTS: usize = 10;

fn gc_reports(dir: &Path) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    let mut files: Vec<_> = rd
        .filter_map(|e| e.ok())
        .map(|e| e.file_name())
        .filter(|n| {
            n.to_string_lossy().starts_with("report-") && n.to_string_lossy().ends_with(".json")
        })
        .collect();
    files.sort_unstable();
    for stale in files.iter().take(files.len().saturating_sub(KEEP_REPORTS)) {
        let _ = std::fs::remove_file(dir.join(stale));
    }
}
