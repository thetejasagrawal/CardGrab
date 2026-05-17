use std::path::PathBuf;
use std::sync::atomic::Ordering;

use tauri::{AppHandle, State};
use uuid::Uuid;

use crate::device::{self, Card, SourceKind, ToolStatus};
use crate::error::{AppError, AppResult};
use crate::journal::{self, ImportEventRow, ImportFileRow, ImportRecord};
use crate::scanner::{self, FileInfo, ScanReport};
use crate::settings::{self, Settings, TemplateRow};
use crate::state::AppState;
use crate::template;
use crate::thumbs;
use crate::transfer::{self, StartImportArgs};

#[tauri::command]
pub fn list_cards(state: tauri::State<'_, AppState>) -> Vec<Card> {
    // Pull from the live watch state so cameras (which need async detection) are included.
    let known = state.known_cards.lock();
    let mut all: Vec<Card> = known.values().cloned().collect();
    all.sort_by(|a, b| {
        let order = |k: SourceKind| match k {
            SourceKind::Sd => 0,
            SourceKind::Camera => 1,
        };
        order(a.kind).cmp(&order(b.kind)).then(a.label.cmp(&b.label))
    });
    all
}

#[tauri::command]
pub async fn scan_card(state: tauri::State<'_, AppState>, mount: String) -> AppResult<ScanReport> {
    if let Some(rest) = mount.strip_prefix("camera://") {
        let port = rest.to_string();
        // Look up camera model from known_cards
        let model = {
            let known = state.known_cards.lock();
            known
                .values()
                .find(|c| c.port.as_deref() == Some(&port))
                .and_then(|c| c.camera_model.clone())
                .unwrap_or_else(|| "Camera".to_string())
        };
        return scanner::scan_camera(&port, &model).await;
    }
    let path = PathBuf::from(&mount);
    tokio::task::spawn_blocking(move || scanner::scan(&path))
        .await
        .map_err(|e| AppError::Other(e.to_string()))?
}

#[tauri::command]
pub fn tool_status() -> ToolStatus {
    device::tool_status()
}

#[derive(serde::Deserialize)]
pub struct PreviewArgs {
    pub pattern: String,
    pub files: Vec<FileInfo>,
    pub count: Option<usize>,
}

#[tauri::command]
pub fn render_template_preview(args: PreviewArgs) -> Vec<String> {
    template::preview_paths(&args.pattern, &args.files, args.count.unwrap_or(5))
}

#[derive(serde::Deserialize)]
pub struct StartImportPayload {
    pub args: StartImportArgs,
    pub report: ScanReport,
}

#[tauri::command]
pub fn start_import(
    app: AppHandle,
    payload: StartImportPayload,
) -> AppResult<String> {
    let import_id = Uuid::new_v4().to_string();
    let id_clone = import_id.clone();
    let app_clone = app.clone();
    // Spawn in background so the command returns immediately and the frontend
    // can navigate to the progress view while progress events stream in.
    tauri::async_runtime::spawn(async move {
        let _ = transfer::run_import(app_clone, id_clone, payload.args, payload.report).await;
    });
    Ok(import_id)
}

#[tauri::command]
pub fn cancel_import(state: State<'_, AppState>, import_id: String) -> AppResult<()> {
    if let Some(ctrl) = state.imports.lock().get(&import_id) {
        ctrl.cancel.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[tauri::command]
pub fn list_imports(state: State<'_, AppState>, limit: Option<usize>) -> AppResult<Vec<ImportRecord>> {
    let conn = state.db.lock();
    journal::list_imports(&conn, limit.unwrap_or(100))
}

#[tauri::command]
pub fn get_import_files(
    state: State<'_, AppState>,
    import_id: String,
) -> AppResult<Vec<ImportFileRow>> {
    let conn = state.db.lock();
    journal::files_for_import(&conn, &import_id)
}

#[tauri::command]
pub fn get_import_events(
    state: State<'_, AppState>,
    import_id: String,
) -> AppResult<Vec<ImportEventRow>> {
    let conn = state.db.lock();
    journal::events_for_import(&conn, &import_id)
}

#[tauri::command]
pub fn reveal_in_finder(path: String) -> Result<(), String> {
    std::process::Command::new("open")
        .arg("-R")
        .arg(&path)
        .status()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn eject_card(mount: String) -> Result<(), String> {
    device::eject(&mount).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pick_destination_dir(app: AppHandle) -> Option<String> {
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog().file().pick_folder(move |path| {
        let _ = tx.send(path.and_then(|p| p.into_path().ok()));
    });
    let result = rx.await.ok().flatten();
    result.map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> AppResult<Settings> {
    settings::load_settings(&state)
}

#[tauri::command]
pub fn set_settings(state: State<'_, AppState>, settings: Settings) -> AppResult<()> {
    settings::save_settings(&state, &settings)
}

#[tauri::command]
pub fn list_templates(state: State<'_, AppState>) -> AppResult<Vec<TemplateRow>> {
    settings::list_templates(&state)
}

#[tauri::command]
pub fn save_template(state: State<'_, AppState>, template: TemplateRow) -> AppResult<()> {
    settings::save_template(&state, &template)
}

#[tauri::command]
pub fn delete_template(state: State<'_, AppState>, id: String) -> AppResult<()> {
    settings::delete_template(&state, &id)
}

#[tauri::command]
pub async fn get_thumbnail(
    state: State<'_, AppState>,
    src: String,
) -> AppResult<String> {
    let data_dir = state.data_dir.clone();
    let src_path = PathBuf::from(&src);
    let path = thumbs::get_thumbnail(&data_dir, &src_path).await?;
    Ok(path.to_string_lossy().to_string())
}
