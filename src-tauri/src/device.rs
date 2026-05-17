use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

use crate::camera;
use crate::state::AppState;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum SourceKind {
    Sd,
    Camera,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: String,
    pub kind: SourceKind,
    /// SD: /Volumes/<label>. Camera: synthetic `camera://<port>`.
    pub mount: String,
    pub label: String,
    pub camera_model: Option<String>,
    pub layout: Vec<String>,
    /// gphoto2 USB port string, e.g. "usb:020,008". Only for cameras.
    pub port: Option<String>,
    pub detected_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolStatus {
    pub gphoto2_installed: bool,
}

const CARD_MARKERS: &[&str] = &["DCIM", "PRIVATE/M4ROOT", "MP_ROOT", "AVCHD", "PRIVATE/AVCHD"];

fn is_camera_card(root: &Path) -> Vec<String> {
    let mut found = Vec::new();
    for m in CARD_MARKERS {
        if root.join(m).exists() {
            found.push((*m).to_string());
        }
    }
    found
}

fn scan_volumes() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/Volumes") {
        for e in entries.flatten() {
            let p = e.path();
            if p.file_name().and_then(|s| s.to_str()) == Some("Macintosh HD") {
                continue;
            }
            if p.is_dir() {
                out.push(p);
            }
        }
    }
    out
}

fn quick_camera_model(root: &Path) -> Option<String> {
    let dcim = root.join("DCIM");
    if !dcim.exists() {
        return None;
    }
    for entry in walkdir::WalkDir::new(&dcim)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .take(50)
    {
        let path = entry.path();
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase());
        if !matches!(ext.as_deref(), Some("jpg") | Some("jpeg")) {
            continue;
        }
        if let Ok(file) = std::fs::File::open(path) {
            let mut reader = std::io::BufReader::new(file);
            if let Ok(exif) = exif::Reader::new().read_from_container(&mut reader) {
                let model = exif
                    .get_field(exif::Tag::Model, exif::In::PRIMARY)
                    .map(|f| f.display_value().to_string().trim_matches('"').to_string());
                if let Some(m) = model {
                    if !m.is_empty() {
                        return Some(m);
                    }
                }
            }
        }
    }
    None
}

fn build_sd_card(mount: &Path, layout: Vec<String>) -> Card {
    let label = mount
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();
    let camera_model = quick_camera_model(mount);
    let id = format!(
        "sd::{}::{}",
        label,
        camera_model.as_deref().unwrap_or("unknown")
    );
    Card {
        id,
        kind: SourceKind::Sd,
        mount: mount.to_string_lossy().to_string(),
        label,
        camera_model,
        layout,
        port: None,
        detected_at: chrono::Utc::now().to_rfc3339(),
    }
}

fn build_camera_card(info: &camera::CameraInfo) -> Card {
    Card {
        id: format!("cam::{}", info.port),
        kind: SourceKind::Camera,
        mount: format!("camera://{}", info.port),
        label: info.model.clone(),
        camera_model: Some(info.model.clone()),
        layout: vec!["PTP".to_string()],
        port: Some(info.port.clone()),
        detected_at: chrono::Utc::now().to_rfc3339(),
    }
}

pub async fn watch_loop(app: AppHandle) {
    let mut tick: u32 = 0;
    loop {
        tokio::time::sleep(Duration::from_millis(1500)).await;
        tick = tick.wrapping_add(1);

        // Discover SD cards in a blocking task with panic isolation. macOS std::fs
        // panics on closedir when a removable device is yanked mid-iteration ("Device
        // not configured"); without catch_unwind that would kill the watch loop and
        // every future card would go undetected.
        let sd_cards: Vec<Card> = tokio::task::spawn_blocking(|| {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let mut cards = Vec::new();
                for vol in scan_volumes() {
                    let layout = is_camera_card(&vol);
                    if layout.is_empty() {
                        continue;
                    }
                    let card_result =
                        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            build_sd_card(&vol, layout.clone())
                        }));
                    if let Ok(card) = card_result {
                        cards.push(card);
                    }
                }
                cards
            }))
            .unwrap_or_default()
        })
        .await
        .unwrap_or_default();

        // Cameras — every 4 ticks (~6s), slower to enumerate via PTP
        let camera_cards: Vec<Card> = if tick % 4 == 0 {
            camera::auto_detect()
                .await
                .unwrap_or_default()
                .iter()
                .map(build_camera_card)
                .collect()
        } else {
            Vec::new()
        };

        let Some(state) = app.try_state::<AppState>() else {
            continue;
        };

        let mut current: HashMap<String, Card> = HashMap::new();
        for card in sd_cards {
            current.insert(card.mount.clone(), card);
        }

        if tick % 4 == 0 {
            for card in camera_cards {
                current.insert(card.mount.clone(), card);
            }
        } else {
            // Carry forward known cameras between checks so they don't blink off.
            let known = state.known_cards.lock();
            for (mount, card) in known.iter() {
                if card.kind == SourceKind::Camera && !current.contains_key(mount) {
                    current.insert(mount.clone(), card.clone());
                }
            }
        }

        let mut known = state.known_cards.lock();
        for (mount, card) in &current {
            if !known.contains_key(mount) {
                let _ = app.emit("card-attached", card);
            }
        }
        let removed: Vec<String> = known
            .keys()
            .filter(|k| !current.contains_key(*k))
            .cloned()
            .collect();
        for mount in &removed {
            if let Some(card) = known.get(mount) {
                let _ = app.emit("card-detached", card.clone());
            }
        }
        *known = current;
    }
}

pub fn eject(mount: &str) -> std::io::Result<()> {
    let status = std::process::Command::new("diskutil")
        .arg("eject")
        .arg(mount)
        .status()?;
    if !status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("diskutil eject exit {status}"),
        ));
    }
    Ok(())
}

pub fn tool_status() -> ToolStatus {
    ToolStatus {
        gphoto2_installed: camera::is_installed(),
    }
}
