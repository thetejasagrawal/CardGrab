use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Semaphore;

use crate::camera;
use crate::error::{AppError, AppResult};
use crate::journal;
use crate::scanner::{FileInfo, ScanReport};
use crate::state::{AppState, ImportControl};
use crate::template;

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CollisionPolicy {
    Skip,
    Rename,
    Overwrite,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StartImportArgs {
    pub card_label: Option<String>,
    pub card_mount: Option<String>,
    pub camera_model: Option<String>,
    pub dest_root: String,
    pub pattern: String,
    pub collision: CollisionPolicy,
    pub verify_hash: bool,
    pub worker_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportProgress {
    pub import_id: String,
    pub file_index: usize,
    pub file_total: usize,
    pub file_name: String,
    pub bytes_done: u64,
    pub bytes_total: u64,
    pub current_file_bytes: u64,
    pub current_file_done: u64,
    pub throughput_bps: u64,
    pub eta_seconds: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportCompleted {
    pub import_id: String,
    pub status: String,
    pub dest_root: String,
    pub file_count: usize,
    pub bytes_total: u64,
    pub failures: usize,
}

fn safety_assert_different_volumes(src: &Path, dst_dir: &Path) -> AppResult<()> {
    // Camera sources have synthetic paths (camera://...) — skip the check.
    if !src.exists() {
        return Ok(());
    }
    // Mock-card mode: source and dest are intentionally on the same volume.
    if std::env::var("CARDGRAB_MOCK_DIR").is_ok() {
        return Ok(());
    }
    let src_meta = std::fs::metadata(src)?;
    let dst_meta = std::fs::metadata(dst_dir)?;
    if src_meta.dev() == dst_meta.dev() {
        return Err(AppError::Safety(format!(
            "refusing to copy to the same volume as the source ({:?})",
            src
        )));
    }
    Ok(())
}

fn resolve_collision(target: &Path, policy: CollisionPolicy) -> AppResult<Option<PathBuf>> {
    if !target.exists() {
        return Ok(Some(target.to_path_buf()));
    }
    match policy {
        CollisionPolicy::Skip => Ok(None),
        CollisionPolicy::Overwrite => Ok(Some(target.to_path_buf())),
        CollisionPolicy::Rename => {
            let parent = target.parent().unwrap_or(Path::new("."));
            let stem = target
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("file");
            let ext = target.extension().and_then(|s| s.to_str()).unwrap_or("");
            for i in 1..10_000 {
                let candidate = if ext.is_empty() {
                    parent.join(format!("{} ({})", stem, i))
                } else {
                    parent.join(format!("{} ({}).{}", stem, i, ext))
                };
                if !candidate.exists() {
                    return Ok(Some(candidate));
                }
            }
            Err(AppError::Other("too many collisions".into()))
        }
    }
}

async fn copy_one(
    file: FileInfo,
    dst_final: PathBuf,
    bytes_total_progress: Arc<AtomicU64>,
    cancel: Arc<AtomicBool>,
    verify_after_copy: bool,
) -> AppResult<()> {
    if let Some(parent) = dst_final.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // ----- PTP camera path: shell out to gphoto2 -----
    if let (Some(port), Some(num)) = (&file.camera_port, file.camera_number) {
        camera::download_one(port, num, &dst_final, cancel.clone()).await?;
        bytes_total_progress.fetch_add(file.bytes, Ordering::Relaxed);
        return Ok(());
    }

    if cancel.load(Ordering::Relaxed) {
        return Err(AppError::Cancelled);
    }

    // ----- Local file path: kernel copy via std::fs::copy. On macOS this lands
    // on fcopyfile(), which uses optimal block sizes and copies in-kernel — far
    // faster than userspace buffered async I/O.
    let src = file.src_abs.clone();
    let dst_clone = dst_final.clone();
    let tmp = dst_final.with_extension(format!(
        "{}.cardgrab.tmp",
        dst_final
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("part")
    ));
    let tmp_clone = tmp.clone();
    let cancel_clone = cancel.clone();

    let copy_result = tokio::task::spawn_blocking(move || -> AppResult<()> {
        if cancel_clone.load(Ordering::Relaxed) {
            return Err(AppError::Cancelled);
        }
        std::fs::copy(&src, &tmp_clone)?;
        if cancel_clone.load(Ordering::Relaxed) {
            let _ = std::fs::remove_file(&tmp_clone);
            return Err(AppError::Cancelled);
        }
        std::fs::rename(&tmp_clone, &dst_clone)?;
        Ok(())
    })
    .await
    .map_err(|e| AppError::Other(e.to_string()))?;

    if let Err(e) = copy_result {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }

    bytes_total_progress.fetch_add(file.bytes, Ordering::Relaxed);

    if let Ok(meta) = std::fs::metadata(&file.src_abs) {
        if let Ok(mt) = meta.modified() {
            let _ = filetime::set_file_mtime(&dst_final, filetime::FileTime::from_system_time(mt));
        }
    }

    if verify_after_copy {
        let src = file.src_abs.clone();
        let dst = dst_final.clone();
        tokio::task::spawn_blocking(move || verify_same_bytes(&src, &dst))
            .await
            .map_err(|e| AppError::Other(e.to_string()))??;
    }
    Ok(())
}

fn verify_same_bytes(src: &Path, dst: &Path) -> AppResult<()> {
    let src_meta = std::fs::metadata(src)?;
    let dst_meta = std::fs::metadata(dst)?;
    if src_meta.len() != dst_meta.len() {
        return Err(AppError::Other(format!(
            "verification failed: size mismatch for {:?}",
            src
        )));
    }

    let mut src_file = std::fs::File::open(src)?;
    let mut dst_file = std::fs::File::open(dst)?;
    let mut src_buf = [0_u8; 1024 * 1024];
    let mut dst_buf = [0_u8; 1024 * 1024];

    loop {
        let src_read = src_file.read(&mut src_buf)?;
        let dst_read = dst_file.read(&mut dst_buf)?;
        if src_read != dst_read || src_buf[..src_read] != dst_buf[..dst_read] {
            return Err(AppError::Other(format!(
                "verification failed: byte mismatch for {:?}",
                src
            )));
        }
        if src_read == 0 {
            return Ok(());
        }
    }
}

pub async fn run_import(
    app: AppHandle,
    import_id: String,
    args: StartImportArgs,
    report: ScanReport,
) -> AppResult<()> {
    let state = app.state::<AppState>();
    let dest_root = PathBuf::from(&args.dest_root);
    std::fs::create_dir_all(&dest_root)?;

    // SAFETY: assert different volumes
    if let Some(first) = report.files.first() {
        safety_assert_different_volumes(&first.src_abs, &dest_root)?;
    }

    {
        let conn = state.db.lock();
        journal::create_import(
            &conn,
            &import_id,
            args.card_label.as_deref(),
            args.card_mount.as_deref(),
            args.camera_model.as_deref(),
            &dest_root,
            &args.pattern,
        )?;
    }

    let cancel = Arc::new(AtomicBool::new(false));
    state.imports.lock().insert(
        import_id.clone(),
        ImportControl {
            cancel: cancel.clone(),
        },
    );

    let bytes_total: u64 = report.files.iter().map(|f| f.bytes).sum();
    let bytes_done = Arc::new(AtomicU64::new(0));
    let workers = args.worker_count.unwrap_or(4).clamp(1, 8);
    let verify_after_copy = args.verify_hash;
    let sem = Arc::new(Semaphore::new(workers));

    // Plan all paths (sequential, fast) so collisions can be resolved deterministically
    let mut planned: Vec<(FileInfo, Option<PathBuf>)> = Vec::with_capacity(report.files.len());
    let mut used: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();
    for f in &report.files {
        let mut target = template::render_full(&dest_root, &args.pattern, f);
        // resolve in-batch collisions too (template can collapse two files to same dst)
        if used.contains(&target) {
            // force a rename with index
            let parent = target.parent().unwrap_or(Path::new(".")).to_path_buf();
            let stem = target
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("file")
                .to_string();
            let ext = target
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            for i in 1..10_000 {
                let cand = if ext.is_empty() {
                    parent.join(format!("{} ({})", stem, i))
                } else {
                    parent.join(format!("{} ({}).{}", stem, i, ext))
                };
                if !used.contains(&cand) {
                    target = cand;
                    break;
                }
            }
        }
        let resolved = resolve_collision(&target, args.collision)?;
        if let Some(p) = &resolved {
            used.insert(p.clone());
        }
        planned.push((f.clone(), resolved));
    }

    let total = planned.len();
    let mut handles = Vec::with_capacity(total);
    let bytes_done_for_progress = bytes_done.clone();
    let progress_total = Arc::new(AtomicU64::new(bytes_total));
    let last_emit = Arc::new(Mutex::new(std::time::Instant::now()));
    let last_bytes = Arc::new(AtomicU64::new(0));
    let started = std::time::Instant::now();

    let failure_count = Arc::new(AtomicU64::new(0));
    let success_count = Arc::new(AtomicU64::new(0));

    for (idx, (file, dst_opt)) in planned.into_iter().enumerate() {
        let sem = sem.clone();
        let cancel = cancel.clone();
        let bytes_done = bytes_done_for_progress.clone();
        let total_bytes_arc = progress_total.clone();
        let last_emit = last_emit.clone();
        let last_bytes = last_bytes.clone();
        let app = app.clone();
        let db = state.db.clone();
        let import_id = import_id.clone();
        let started = started;
        let failure_count = failure_count.clone();
        let success_count = success_count.clone();
        let verify_after_copy = verify_after_copy;

        let handle = tokio::spawn(async move {
            let _permit = match sem.acquire_owned().await {
                Ok(p) => p,
                Err(_) => return,
            };
            if cancel.load(Ordering::Relaxed) {
                return;
            }

            // Skipped (collision = Skip and target exists)
            let Some(dst) = dst_opt else {
                let conn = db.lock();
                let _ = journal::log_file(&conn, &import_id, &file, Path::new(""), "skipped", None);
                bytes_done.fetch_add(file.bytes, Ordering::Relaxed);
                emit_progress(
                    &app,
                    &import_id,
                    idx,
                    total,
                    &file.orig_name,
                    bytes_done.load(Ordering::Relaxed),
                    total_bytes_arc.load(Ordering::Relaxed),
                    file.bytes,
                    file.bytes,
                    &last_emit,
                    &last_bytes,
                    started,
                );
                return;
            };

            match copy_one(
                file.clone(),
                dst.clone(),
                bytes_done.clone(),
                cancel.clone(),
                verify_after_copy,
            )
            .await
            {
                Ok(_) => {
                    success_count.fetch_add(1, Ordering::Relaxed);
                    let conn = db.lock();
                    let _ = journal::log_file(&conn, &import_id, &file, &dst, "ok", None);
                }
                Err(e) => {
                    failure_count.fetch_add(1, Ordering::Relaxed);
                    let conn = db.lock();
                    let _ = journal::log_file(
                        &conn,
                        &import_id,
                        &file,
                        &dst,
                        if matches!(e, AppError::Cancelled) {
                            "cancelled"
                        } else {
                            "failed"
                        },
                        Some(&e.to_string()),
                    );
                }
            }

            emit_progress(
                &app,
                &import_id,
                idx,
                total,
                &file.orig_name,
                bytes_done.load(Ordering::Relaxed),
                total_bytes_arc.load(Ordering::Relaxed),
                file.bytes,
                file.bytes,
                &last_emit,
                &last_bytes,
                started,
            );
        });
        handles.push(handle);
    }

    for h in handles {
        let _ = h.await;
    }

    let was_cancelled = cancel.load(Ordering::Relaxed);
    let failures = failure_count.load(Ordering::Relaxed) as usize;
    let successes = success_count.load(Ordering::Relaxed) as usize;

    let status = if was_cancelled {
        "cancelled"
    } else if failures > 0 {
        "completed_with_errors"
    } else {
        "completed"
    };
    {
        let conn = state.db.lock();
        let _ = journal::finalize_import(&conn, &import_id, status, successes, bytes_done.load(Ordering::Relaxed));
    }
    state.imports.lock().remove(&import_id);

    let _ = app.emit(
        "import-complete",
        ImportCompleted {
            import_id,
            status: status.to_string(),
            dest_root: dest_root.to_string_lossy().to_string(),
            file_count: successes,
            bytes_total: bytes_done.load(Ordering::Relaxed),
            failures,
        },
    );

    Ok(())
}

fn emit_progress(
    app: &AppHandle,
    import_id: &str,
    file_index: usize,
    file_total: usize,
    file_name: &str,
    bytes_done: u64,
    bytes_total: u64,
    current_file_bytes: u64,
    current_file_done: u64,
    last_emit: &Mutex<std::time::Instant>,
    last_bytes: &AtomicU64,
    started: std::time::Instant,
) {
    let mut le = last_emit.lock();
    let elapsed = le.elapsed();
    if elapsed < std::time::Duration::from_millis(100) && file_index + 1 < file_total {
        return;
    }
    *le = std::time::Instant::now();
    drop(le);

    let now_b = bytes_done;
    let dt_total = started.elapsed().as_secs_f64().max(0.001);
    let throughput = (now_b as f64 / dt_total) as u64;
    last_bytes.store(now_b, Ordering::Relaxed);
    let remaining = bytes_total.saturating_sub(now_b);
    let eta = if throughput > 0 {
        remaining / throughput
    } else {
        0
    };
    let _ = app.emit(
        "import-progress",
        ImportProgress {
            import_id: import_id.to_string(),
            file_index,
            file_total,
            file_name: file_name.to_string(),
            bytes_done: now_b,
            bytes_total,
            current_file_bytes,
            current_file_done,
            throughput_bps: throughput,
            eta_seconds: eta,
        },
    );
}
