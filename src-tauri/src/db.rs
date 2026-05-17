use std::path::Path;

use rusqlite::{params, Connection};

use crate::error::AppResult;

pub fn open(path: &Path) -> AppResult<Connection> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    migrate(&conn)?;
    seed_defaults(&conn)?;
    Ok(conn)
}

fn migrate(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS settings (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS templates (
            id         TEXT PRIMARY KEY,
            name       TEXT NOT NULL,
            pattern    TEXT NOT NULL,
            is_default INTEGER NOT NULL DEFAULT 0,
            built_in   INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS imports (
            id              TEXT PRIMARY KEY,
            started_at      TEXT NOT NULL,
            finished_at     TEXT,
            status          TEXT NOT NULL,
            card_label      TEXT,
            card_mount      TEXT,
            camera_model    TEXT,
            dest_root       TEXT NOT NULL,
            template_pattern TEXT NOT NULL,
            file_count      INTEGER NOT NULL DEFAULT 0,
            bytes           INTEGER NOT NULL DEFAULT 0,
            notes           TEXT
        );

        CREATE TABLE IF NOT EXISTS import_files (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            import_id   TEXT NOT NULL,
            src_rel     TEXT NOT NULL,
            src_abs     TEXT NOT NULL,
            dst_abs     TEXT NOT NULL,
            bytes       INTEGER NOT NULL,
            mtime       TEXT,
            kind        TEXT NOT NULL,
            status      TEXT NOT NULL,
            error_msg   TEXT,
            FOREIGN KEY (import_id) REFERENCES imports(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_import_files_import ON import_files(import_id);

        CREATE TABLE IF NOT EXISTS import_events (
            id        INTEGER PRIMARY KEY AUTOINCREMENT,
            import_id TEXT NOT NULL,
            ts        TEXT NOT NULL,
            level     TEXT NOT NULL,
            message   TEXT NOT NULL,
            FOREIGN KEY (import_id) REFERENCES imports(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_import_events_import ON import_events(import_id);
        "#,
    )?;
    Ok(())
}

fn seed_defaults(conn: &Connection) -> AppResult<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM templates", [], |r| r.get(0))?;
    if count == 0 {
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO templates (id, name, pattern, is_default, built_in, created_at)
             VALUES (?1, ?2, ?3, 1, 1, ?4)",
            params!["auto-date", "By date", "{date}", now],
        )?;
        conn.execute(
            "INSERT INTO templates (id, name, pattern, is_default, built_in, created_at)
             VALUES (?1, ?2, ?3, 0, 1, ?4)",
            params![
                "auto-camera-date",
                "By camera + date",
                "{camera}/{date}",
                chrono::Utc::now().to_rfc3339()
            ],
        )?;
        conn.execute(
            "INSERT INTO templates (id, name, pattern, is_default, built_in, created_at)
             VALUES (?1, ?2, ?3, 0, 1, ?4)",
            params![
                "flat",
                "Flat (no folders)",
                "{orig_name}",
                chrono::Utc::now().to_rfc3339()
            ],
        )?;
    } else {
        // Idempotent fix-up for existing installs: simplify built-in templates
        // to the new dead-simple defaults.
        conn.execute(
            "UPDATE templates SET name = 'By date', pattern = '{date}'
             WHERE id = 'auto-date' AND built_in = 1",
            [],
        )?;
        conn.execute(
            "UPDATE templates SET name = 'By camera + date', pattern = '{camera}/{date}'
             WHERE id = 'auto-camera-date' AND built_in = 1",
            [],
        )?;
        conn.execute(
            "UPDATE templates SET name = 'Flat (no folders)'
             WHERE id = 'flat' AND built_in = 1",
            [],
        )?;
    }

    let s_count: i64 = conn.query_row("SELECT COUNT(*) FROM settings", [], |r| r.get(0))?;
    if s_count == 0 {
        let default_dest = dirs::picture_dir()
            .map(|p| p.join("cardgrab").to_string_lossy().to_string())
            .unwrap_or_else(|| "~/Pictures/cardgrab".to_string());
        for (k, v) in [
            ("default_dest", default_dest.as_str()),
            ("collision_policy", "rename"),
            ("verify_hash", "false"),
            ("worker_count", "3"),
        ] {
            conn.execute(
                "INSERT INTO settings(key, value) VALUES (?1, ?2)",
                params![k, v],
            )?;
        }
    }

    Ok(())
}
