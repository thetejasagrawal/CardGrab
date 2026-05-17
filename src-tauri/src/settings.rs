use std::collections::HashMap;

use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub default_dest: String,
    pub collision_policy: String,
    pub verify_hash: bool,
    pub worker_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateRow {
    pub id: String,
    pub name: String,
    pub pattern: String,
    pub is_default: bool,
    pub built_in: bool,
    pub created_at: String,
}

pub fn load_settings(state: &AppState) -> AppResult<Settings> {
    let conn = state.db.lock();
    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
    let mut map: HashMap<String, String> = HashMap::new();
    for r in rows.flatten() {
        map.insert(r.0, r.1);
    }
    Ok(Settings {
        default_dest: map.get("default_dest").cloned().unwrap_or_default(),
        collision_policy: map
            .get("collision_policy")
            .cloned()
            .unwrap_or_else(|| "rename".into()),
        verify_hash: map
            .get("verify_hash")
            .map(|v| v == "true")
            .unwrap_or(false),
        worker_count: map
            .get("worker_count")
            .and_then(|v| v.parse().ok())
            .unwrap_or(3),
    })
}

pub fn save_settings(state: &AppState, s: &Settings) -> AppResult<()> {
    let conn = state.db.lock();
    let pairs: [(&str, String); 4] = [
        ("default_dest", s.default_dest.clone()),
        ("collision_policy", s.collision_policy.clone()),
        ("verify_hash", s.verify_hash.to_string()),
        ("worker_count", s.worker_count.to_string()),
    ];
    for (k, v) in pairs {
        conn.execute(
            "INSERT INTO settings(key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![k, v],
        )?;
    }
    Ok(())
}

pub fn list_templates(state: &AppState) -> AppResult<Vec<TemplateRow>> {
    let conn = state.db.lock();
    let mut stmt = conn.prepare(
        "SELECT id, name, pattern, is_default, built_in, created_at FROM templates ORDER BY built_in DESC, name ASC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(TemplateRow {
            id: r.get(0)?,
            name: r.get(1)?,
            pattern: r.get(2)?,
            is_default: r.get::<_, i64>(3)? != 0,
            built_in: r.get::<_, i64>(4)? != 0,
            created_at: r.get(5)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn save_template(state: &AppState, t: &TemplateRow) -> AppResult<()> {
    let conn = state.db.lock();
    if t.is_default {
        conn.execute("UPDATE templates SET is_default = 0", [])?;
    }
    conn.execute(
        "INSERT INTO templates(id, name, pattern, is_default, built_in, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            pattern = excluded.pattern,
            is_default = excluded.is_default",
        params![
            t.id,
            t.name,
            t.pattern,
            t.is_default as i64,
            t.built_in as i64,
            t.created_at
        ],
    )?;
    Ok(())
}

pub fn delete_template(state: &AppState, id: &str) -> AppResult<()> {
    let conn = state.db.lock();
    conn.execute(
        "DELETE FROM templates WHERE id = ?1 AND built_in = 0",
        params![id],
    )?;
    Ok(())
}
