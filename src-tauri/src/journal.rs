use std::path::{Path, PathBuf};

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::scanner::{FileInfo, MediaKind};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportRecord {
    pub id: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub status: String,
    pub card_label: Option<String>,
    pub card_mount: Option<String>,
    pub camera_model: Option<String>,
    pub dest_root: String,
    pub template_pattern: String,
    pub file_count: i64,
    pub bytes: i64,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportFileRow {
    pub id: i64,
    pub import_id: String,
    pub src_rel: String,
    pub src_abs: String,
    pub dst_abs: String,
    pub bytes: i64,
    pub mtime: Option<String>,
    pub kind: String,
    pub status: String,
    pub error_msg: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportEventRow {
    pub id: i64,
    pub import_id: String,
    pub ts: String,
    pub level: String,
    pub message: String,
}

pub fn create_import(
    conn: &Connection,
    id: &str,
    card_label: Option<&str>,
    card_mount: Option<&str>,
    camera_model: Option<&str>,
    dest_root: &Path,
    pattern: &str,
) -> AppResult<()> {
    conn.execute(
        "INSERT INTO imports (id, started_at, status, card_label, card_mount, camera_model, dest_root, template_pattern)
         VALUES (?1, ?2, 'running', ?3, ?4, ?5, ?6, ?7)",
        params![
            id,
            Utc::now().to_rfc3339(),
            card_label,
            card_mount,
            camera_model,
            dest_root.to_string_lossy(),
            pattern,
        ],
    )?;
    log_event(conn, id, "info", "import started")?;
    Ok(())
}

pub fn log_file(
    conn: &Connection,
    import_id: &str,
    file: &FileInfo,
    dst: &Path,
    status: &str,
    err: Option<&str>,
) -> AppResult<()> {
    conn.execute(
        "INSERT INTO import_files (import_id, src_rel, src_abs, dst_abs, bytes, mtime, kind, status, error_msg)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            import_id,
            file.src_rel.to_string_lossy(),
            file.src_abs.to_string_lossy(),
            dst.to_string_lossy(),
            file.bytes as i64,
            file.mtime.map(|t| t.to_rfc3339()),
            media_kind_name(file.kind),
            status,
            err,
        ],
    )?;
    Ok(())
}

pub fn log_event(conn: &Connection, import_id: &str, level: &str, message: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO import_events (import_id, ts, level, message) VALUES (?1, ?2, ?3, ?4)",
        params![import_id, Utc::now().to_rfc3339(), level, message],
    )?;
    Ok(())
}

pub fn finalize_import(
    conn: &Connection,
    id: &str,
    status: &str,
    file_count: usize,
    bytes: u64,
) -> AppResult<()> {
    conn.execute(
        "UPDATE imports SET finished_at = ?1, status = ?2, file_count = ?3, bytes = ?4 WHERE id = ?5",
        params![
            Utc::now().to_rfc3339(),
            status,
            file_count as i64,
            bytes as i64,
            id
        ],
    )?;
    log_event(conn, id, "info", &format!("import {status}"))?;
    Ok(())
}

pub fn list_imports(conn: &Connection, limit: usize) -> AppResult<Vec<ImportRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, started_at, finished_at, status, card_label, card_mount, camera_model,
                dest_root, template_pattern, file_count, bytes, notes
         FROM imports ORDER BY started_at DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit as i64], |r| {
        Ok(ImportRecord {
            id: r.get(0)?,
            started_at: r.get(1)?,
            finished_at: r.get(2)?,
            status: r.get(3)?,
            card_label: r.get(4)?,
            card_mount: r.get(5)?,
            camera_model: r.get(6)?,
            dest_root: r.get(7)?,
            template_pattern: r.get(8)?,
            file_count: r.get(9)?,
            bytes: r.get(10)?,
            notes: r.get(11)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn files_for_import(conn: &Connection, import_id: &str) -> AppResult<Vec<ImportFileRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, import_id, src_rel, src_abs, dst_abs, bytes, mtime, kind, status, error_msg
         FROM import_files WHERE import_id = ?1 ORDER BY id ASC",
    )?;
    let rows = stmt.query_map(params![import_id], |r| {
        Ok(ImportFileRow {
            id: r.get(0)?,
            import_id: r.get(1)?,
            src_rel: r.get(2)?,
            src_abs: r.get(3)?,
            dst_abs: r.get(4)?,
            bytes: r.get(5)?,
            mtime: r.get(6)?,
            kind: r.get(7)?,
            status: r.get(8)?,
            error_msg: r.get(9)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn events_for_import(conn: &Connection, import_id: &str) -> AppResult<Vec<ImportEventRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, import_id, ts, level, message FROM import_events
         WHERE import_id = ?1 ORDER BY id ASC",
    )?;
    let rows = stmt.query_map(params![import_id], |r| {
        Ok(ImportEventRow {
            id: r.get(0)?,
            import_id: r.get(1)?,
            ts: r.get(2)?,
            level: r.get(3)?,
            message: r.get(4)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

fn media_kind_name(k: MediaKind) -> &'static str {
    match k {
        MediaKind::Photo => "photo",
        MediaKind::Raw => "raw",
        MediaKind::Video => "video",
        MediaKind::Audio => "audio",
        MediaKind::Sidecar => "sidecar",
        MediaKind::Other => "other",
    }
}

#[allow(dead_code)]
pub fn export_csv(conn: &Connection, import_id: &str, dst: &Path) -> AppResult<PathBuf> {
    let rows = files_for_import(conn, import_id)?;
    let mut out = String::from("src_abs,dst_abs,bytes,kind,status,error\n");
    for r in rows {
        out.push_str(&format!(
            "\"{}\",\"{}\",{},{},{},\"{}\"\n",
            r.src_abs.replace('"', "\"\""),
            r.dst_abs.replace('"', "\"\""),
            r.bytes,
            r.kind,
            r.status,
            r.error_msg.unwrap_or_default().replace('"', "\"\"")
        ));
    }
    std::fs::write(dst, out)?;
    Ok(dst.to_path_buf())
}
