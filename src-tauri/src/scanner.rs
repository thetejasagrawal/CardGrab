use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::camera::{self, CameraFile};
use crate::error::AppResult;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum MediaKind {
    Photo,
    Raw,
    Video,
    Audio,
    Sidecar,
    Other,
}

impl MediaKind {
    pub fn folder_name(&self) -> &'static str {
        match self {
            MediaKind::Photo => "Photos",
            MediaKind::Raw => "Raw",
            MediaKind::Video => "Videos",
            MediaKind::Audio => "Audio",
            MediaKind::Sidecar => "Sidecars",
            MediaKind::Other => "Other",
        }
    }
}

fn classify(path: &Path) -> MediaKind {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "jpg" | "jpeg" | "heic" | "heif" | "png" | "tif" | "tiff" | "bmp" | "webp" => {
            MediaKind::Photo
        }
        "arw" | "arq" | "dng" | "cr2" | "cr3" | "nef" | "raf" | "rw2" | "orf" | "srw" | "pef"
        | "raw" => MediaKind::Raw,
        "mp4" | "mxf" | "mov" | "mts" | "m2ts" | "avi" | "mkv" | "m4v" | "mpg" | "mpeg" | "3gp" => {
            MediaKind::Video
        }
        "wav" | "mp3" | "m4a" | "aac" | "flac" | "ogg" | "wma" => MediaKind::Audio,
        "xml" | "xmp" | "thm" | "lrv" | "modd" | "moff" => MediaKind::Sidecar,
        _ => MediaKind::Other,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub src_abs: PathBuf,
    pub src_rel: PathBuf,
    pub orig_name: String,
    pub ext: String,
    pub bytes: u64,
    pub kind: MediaKind,
    pub mtime: Option<DateTime<Utc>>,
    pub shot_at: Option<DateTime<Utc>>,
    pub camera_model: Option<String>,
    pub lens: Option<String>,
    /// Set when the source is a PTP camera. Tells the transfer engine to
    /// use gphoto2 instead of a regular file copy.
    pub camera_port: Option<String>,
    pub camera_number: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    pub root: PathBuf,
    pub files: Vec<FileInfo>,
    pub total_bytes: u64,
    pub photo_count: usize,
    pub raw_count: usize,
    pub video_count: usize,
    pub audio_count: usize,
    pub sidecar_count: usize,
    pub other_count: usize,
    #[serde(default)]
    pub permission_denied: bool,
    pub camera_model: Option<String>,
    pub earliest: Option<DateTime<Utc>>,
    pub latest: Option<DateTime<Utc>>,
}

fn is_system_junk(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    matches!(
        name,
        ".Spotlight-V100"
            | ".fseventsd"
            | ".Trashes"
            | ".DS_Store"
            | ".TemporaryItems"
            | ".DocumentRevisions-V100"
            | "System Volume Information"
    )
}

fn read_exif(path: &Path) -> Option<(Option<DateTime<Utc>>, Option<String>, Option<String>)> {
    let file = std::fs::File::open(path).ok()?;
    let mut reader = std::io::BufReader::new(file);
    let exif = exif::Reader::new().read_from_container(&mut reader).ok()?;

    let shot_at = exif
        .get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY)
        .or_else(|| exif.get_field(exif::Tag::DateTime, exif::In::PRIMARY))
        .and_then(|f| {
            let s = f.display_value().to_string();
            // EXIF date format: "YYYY-MM-DD HH:MM:SS" (with quotes from display_value)
            let s = s.trim_matches('"');
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                .ok()
                .map(|naive| DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc))
        });

    let camera = exif
        .get_field(exif::Tag::Model, exif::In::PRIMARY)
        .map(|f| f.display_value().to_string().trim_matches('"').to_string())
        .filter(|s| !s.is_empty());

    let lens = exif
        .get_field(exif::Tag::LensModel, exif::In::PRIMARY)
        .map(|f| f.display_value().to_string().trim_matches('"').to_string())
        .filter(|s| !s.is_empty());

    Some((shot_at, camera, lens))
}

pub fn scan(root: &Path) -> AppResult<ScanReport> {
    let mut files: Vec<FileInfo> = Vec::new();
    let mut camera_model: Option<String> = None;
    let mut earliest: Option<DateTime<Utc>> = None;
    let mut latest: Option<DateTime<Utc>> = None;
    let mut walk_errors: u32 = 0;
    let mut entries_seen: u32 = 0;
    let mut permission_denied = false;

    for entry in WalkDir::new(root).into_iter().filter_entry(|e| {
        // skip system junk at any depth
        !is_system_junk(e.path())
    }) {
        entries_seen += 1;
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                walk_errors += 1;
                if let Some(io) = e.io_error() {
                    if matches!(io.kind(), std::io::ErrorKind::PermissionDenied)
                        || io.raw_os_error() == Some(1)
                    {
                        permission_denied = true;
                    }
                }
                if walk_errors <= 3 {
                    eprintln!("[scanner] walk error at {:?}: {}", root, e);
                }
                continue;
            }
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let rel = path.strip_prefix(root).unwrap_or(path).to_path_buf();
        let kind = classify(path);

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let bytes = metadata.len();
        let mtime = metadata.modified().ok().and_then(|t| {
            t.duration_since(std::time::UNIX_EPOCH)
                .ok()
                .map(|d| DateTime::<Utc>::from_timestamp(d.as_secs() as i64, d.subsec_nanos()))
                .flatten()
        });

        let (shot_at, camera, lens) = if matches!(kind, MediaKind::Photo | MediaKind::Raw) {
            read_exif(path).unwrap_or((None, None, None))
        } else {
            (None, None, None)
        };

        if camera_model.is_none() {
            camera_model = camera.clone();
        }

        let when = shot_at.or(mtime);
        if let Some(t) = when {
            earliest = Some(earliest.map(|e| e.min(t)).unwrap_or(t));
            latest = Some(latest.map(|l| l.max(t)).unwrap_or(t));
        }

        let orig_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();

        files.push(FileInfo {
            src_abs: path.to_path_buf(),
            src_rel: rel,
            orig_name,
            ext,
            bytes,
            kind,
            mtime,
            shot_at,
            camera_model: camera,
            lens,
            camera_port: None,
            camera_number: None,
        });
    }

    eprintln!(
        "[scanner] {:?} → entries_seen={} files={} walk_errors={}",
        root,
        entries_seen,
        files.len(),
        walk_errors
    );

    let total_bytes: u64 = files.iter().map(|f| f.bytes).sum();
    let photo_count = files.iter().filter(|f| f.kind == MediaKind::Photo).count();
    let raw_count = files.iter().filter(|f| f.kind == MediaKind::Raw).count();
    let video_count = files.iter().filter(|f| f.kind == MediaKind::Video).count();
    let audio_count = files.iter().filter(|f| f.kind == MediaKind::Audio).count();
    let sidecar_count = files.iter().filter(|f| f.kind == MediaKind::Sidecar).count();
    let other_count = files.iter().filter(|f| f.kind == MediaKind::Other).count();

    Ok(ScanReport {
        root: root.to_path_buf(),
        files,
        total_bytes,
        photo_count,
        raw_count,
        video_count,
        audio_count,
        sidecar_count,
        other_count,
        permission_denied,
        camera_model,
        earliest,
        latest,
    })
}

/// Build a ScanReport from a PTP camera (no EXIF — metadata comes from gphoto2 listing).
pub async fn scan_camera(port: &str, camera_model: &str) -> AppResult<ScanReport> {
    let listing = camera::list_files(port).await?;
    let files: Vec<FileInfo> = listing
        .iter()
        .map(|f: &CameraFile| {
            let ext = f
                .name
                .rsplit('.')
                .next()
                .map(|s| s.to_ascii_lowercase())
                .unwrap_or_default();
            FileInfo {
                src_abs: PathBuf::from(format!("camera://{}/{}", port, f.name)),
                src_rel: PathBuf::from(&f.name),
                orig_name: f.name.clone(),
                ext,
                bytes: f.bytes,
                kind: camera::classify_by_name(&f.name),
                mtime: None,
                shot_at: None,
                camera_model: Some(camera_model.to_string()),
                lens: None,
                camera_port: Some(port.to_string()),
                camera_number: Some(f.number),
            }
        })
        .collect();

    let total_bytes: u64 = files.iter().map(|f| f.bytes).sum();
    let photo_count = files.iter().filter(|f| f.kind == MediaKind::Photo).count();
    let raw_count = files.iter().filter(|f| f.kind == MediaKind::Raw).count();
    let video_count = files.iter().filter(|f| f.kind == MediaKind::Video).count();
    let audio_count = files.iter().filter(|f| f.kind == MediaKind::Audio).count();
    let sidecar_count = files.iter().filter(|f| f.kind == MediaKind::Sidecar).count();
    let other_count = files.iter().filter(|f| f.kind == MediaKind::Other).count();

    Ok(ScanReport {
        root: PathBuf::from(format!("camera://{}", port)),
        files,
        total_bytes,
        photo_count,
        raw_count,
        video_count,
        audio_count,
        sidecar_count,
        other_count,
        permission_denied: false,
        camera_model: Some(camera_model.to_string()),
        earliest: None,
        latest: None,
    })
}
