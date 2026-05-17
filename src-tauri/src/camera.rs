use std::path::Path;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraInfo {
    pub model: String,
    pub port: String, // e.g. "usb:020,008"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraFile {
    pub number: u32,
    pub folder: String,
    pub name: String,
    pub bytes: u64,
    pub mime: String,
}

pub fn is_installed() -> bool {
    std::process::Command::new("which")
        .arg("gphoto2")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// macOS holds PTP cameras via the `PTPCamera` daemon. Kill it before talking
/// to the device so gphoto2 can acquire the USB interface.
fn release_macos_lock() {
    let _ = std::process::Command::new("killall")
        .arg("PTPCamera")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

pub async fn auto_detect() -> AppResult<Vec<CameraInfo>> {
    if !is_installed() {
        return Ok(Vec::new());
    }
    release_macos_lock();
    let out = Command::new("gphoto2").arg("--auto-detect").output().await?;
    if !out.status.success() {
        return Ok(Vec::new());
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut cameras = Vec::new();
    for (i, line) in text.lines().enumerate() {
        // skip "Model     Port" + "----" separator
        if i < 2 {
            continue;
        }
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(idx) = trimmed.rfind(|c: char| c.is_whitespace()) {
            let port = trimmed[idx..].trim();
            let model = trimmed[..idx].trim();
            if port.starts_with("usb:") {
                cameras.push(CameraInfo {
                    model: model.to_string(),
                    port: port.to_string(),
                });
            }
        }
    }
    Ok(cameras)
}

pub async fn list_files(port: &str) -> AppResult<Vec<CameraFile>> {
    release_macos_lock();
    let out = Command::new("gphoto2")
        .arg("--port")
        .arg(port)
        .arg("--list-files")
        .output()
        .await?;
    if !out.status.success() {
        let msg = String::from_utf8_lossy(&out.stderr).to_string();
        return Err(AppError::Other(format!("gphoto2 list-files failed: {msg}")));
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut current_folder = String::from("/");
    let mut files = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("There are ") {
            if let Some(start) = trimmed.find("in folder '") {
                let rest = &trimmed[start + 11..];
                if let Some(end) = rest.rfind('\'') {
                    current_folder = rest[..end].to_string();
                }
            }
        } else if trimmed.starts_with('#') {
            // Format: #N  filename  rd  size unit  mime
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() < 5 {
                continue;
            }
            let number: u32 = match parts[0].trim_start_matches('#').parse() {
                Ok(n) => n,
                Err(_) => continue,
            };
            let name = parts[1].to_string();
            let size_val: u64 = parts[3].parse().unwrap_or(0);
            let bytes = match parts[4] {
                "KB" => size_val * 1024,
                "MB" => size_val * 1024 * 1024,
                "GB" => size_val * 1024 * 1024 * 1024,
                _ => size_val,
            };
            let mime = parts.get(5).map(|s| s.to_string()).unwrap_or_default();
            files.push(CameraFile {
                number,
                folder: current_folder.clone(),
                name,
                bytes,
                mime,
            });
        }
    }
    Ok(files)
}

pub async fn download_one(
    port: &str,
    file_number: u32,
    dest_path: &Path,
    cancel: Arc<AtomicBool>,
) -> AppResult<()> {
    if cancel.load(Ordering::Relaxed) {
        return Err(AppError::Cancelled);
    }
    release_macos_lock();
    let parent = dest_path
        .parent()
        .ok_or_else(|| AppError::Other("destination has no parent dir".into()))?;
    std::fs::create_dir_all(parent)?;
    let filename = dest_path
        .file_name()
        .ok_or_else(|| AppError::Other("destination has no filename".into()))?;

    let status = Command::new("gphoto2")
        .current_dir(parent)
        .arg("--port")
        .arg(port)
        .arg("--force-overwrite")
        .arg("--filename")
        .arg(filename)
        .arg("--get-file")
        .arg(file_number.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await?;
    if !status.success() {
        return Err(AppError::Other(format!("gphoto2 get-file exit {status}")));
    }
    Ok(())
}

/// Cheap classification from the filename — used when building FileInfo
/// without EXIF (camera scan doesn't have EXIF until download).
pub fn classify_by_name(name: &str) -> crate::scanner::MediaKind {
    let ext = name
        .rsplit('.')
        .next()
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "jpg" | "jpeg" | "heic" | "heif" | "png" | "tif" | "tiff" => {
            crate::scanner::MediaKind::Photo
        }
        "arw" | "arq" | "dng" | "cr2" | "cr3" | "nef" | "raf" | "rw2" | "orf" | "srw" | "pef"
        | "raw" => crate::scanner::MediaKind::Raw,
        "mp4" | "mxf" | "mov" | "mts" | "m2ts" | "avi" | "mkv" | "m4v" => {
            crate::scanner::MediaKind::Video
        }
        "wav" | "mp3" | "m4a" | "aac" | "flac" => crate::scanner::MediaKind::Audio,
        "xml" | "xmp" | "thm" | "lrv" => crate::scanner::MediaKind::Sidecar,
        _ => crate::scanner::MediaKind::Other,
    }
}
