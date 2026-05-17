use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use once_cell::sync::Lazy;
use tokio::process::Command;
use tokio::sync::Semaphore;

use crate::error::{AppError, AppResult};

static QL_SEMAPHORE: Lazy<Arc<Semaphore>> = Lazy::new(|| Arc::new(Semaphore::new(4)));

const THUMB_SIZE: u32 = 320;

fn cache_key(src: &Path, size: u64, mtime_secs: i64) -> String {
    let mut h = DefaultHasher::new();
    src.to_string_lossy().hash(&mut h);
    size.hash(&mut h);
    mtime_secs.hash(&mut h);
    THUMB_SIZE.hash(&mut h);
    format!("{:016x}", h.finish())
}

fn cache_root(data_dir: &Path) -> PathBuf {
    data_dir.join("thumbs")
}

pub async fn get_thumbnail(data_dir: &Path, src: &Path) -> AppResult<PathBuf> {
    if !src.exists() {
        return Err(AppError::Other("source file not found".into()));
    }
    let meta = std::fs::metadata(src)?;
    let size = meta.len();
    let mtime_secs = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let key = cache_key(src, size, mtime_secs);
    let root = cache_root(data_dir);
    std::fs::create_dir_all(&root)?;
    let final_path = root.join(format!("{key}.png"));

    if final_path.exists() {
        return Ok(final_path);
    }

    let _permit = QL_SEMAPHORE
        .clone()
        .acquire_owned()
        .await
        .map_err(|e| AppError::Other(e.to_string()))?;

    // qlmanage writes <input-basename>.png into the output directory, so we give it
    // a dedicated tmp dir to avoid collisions across concurrent calls.
    let tmp_dir = root.join("tmp").join(&key);
    std::fs::create_dir_all(&tmp_dir)?;

    let status = Command::new("qlmanage")
        .arg("-t")
        .arg("-s")
        .arg(THUMB_SIZE.to_string())
        .arg("-o")
        .arg(&tmp_dir)
        .arg(src)
        .output()
        .await;

    let basename = src
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("thumb");
    let produced = tmp_dir.join(format!("{basename}.png"));

    let ok = match status {
        Ok(o) if o.status.success() => produced.exists(),
        Ok(_) => produced.exists(),
        Err(_) => false,
    };

    if ok {
        let _ = std::fs::rename(&produced, &final_path);
        let _ = std::fs::remove_dir_all(&tmp_dir);
        Ok(final_path)
    } else {
        let _ = std::fs::remove_dir_all(&tmp_dir);
        Err(AppError::Other("thumbnail generation failed".into()))
    }
}
