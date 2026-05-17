import { invoke, convertFileSrc } from '@tauri-apps/api/core';
import { listen, type Event as TauriEvent } from '@tauri-apps/api/event';

// ---------- Types ----------

export type MediaKind = 'photo' | 'raw' | 'video' | 'audio' | 'sidecar' | 'other';
export type SourceKind = 'sd' | 'camera';

export interface Card {
  id: string;
  kind: SourceKind;
  mount: string;
  label: string;
  camera_model: string | null;
  layout: string[];
  port: string | null;
  detected_at: string;
}

export interface ToolStatus {
  gphoto2_installed: boolean;
}

export interface FileInfo {
  src_abs: string;
  src_rel: string;
  orig_name: string;
  ext: string;
  bytes: number;
  kind: MediaKind;
  mtime: string | null;
  shot_at: string | null;
  camera_model: string | null;
  lens: string | null;
  camera_port: string | null;
  camera_number: number | null;
}

export interface ScanReport {
  root: string;
  files: FileInfo[];
  total_bytes: number;
  photo_count: number;
  raw_count: number;
  video_count: number;
  audio_count: number;
  sidecar_count: number;
  other_count: number;
  camera_model: string | null;
  earliest: string | null;
  latest: string | null;
}

export interface Settings {
  default_dest: string;
  collision_policy: string;
  verify_hash: boolean;
  worker_count: number;
}

export interface TemplateRow {
  id: string;
  name: string;
  pattern: string;
  is_default: boolean;
  built_in: boolean;
  created_at: string;
}

export interface ImportRecord {
  id: string;
  started_at: string;
  finished_at: string | null;
  status: string;
  card_label: string | null;
  card_mount: string | null;
  camera_model: string | null;
  dest_root: string;
  template_pattern: string;
  file_count: number;
  bytes: number;
  notes: string | null;
}

export interface ImportFileRow {
  id: number;
  import_id: string;
  src_rel: string;
  src_abs: string;
  dst_abs: string;
  bytes: number;
  mtime: string | null;
  kind: string;
  status: string;
  error_msg: string | null;
}

export interface ImportEventRow {
  id: number;
  import_id: string;
  ts: string;
  level: string;
  message: string;
}

export interface ImportProgress {
  import_id: string;
  file_index: number;
  file_total: number;
  file_name: string;
  bytes_done: number;
  bytes_total: number;
  current_file_bytes: number;
  current_file_done: number;
  throughput_bps: number;
  eta_seconds: number;
}

export interface ImportCompleted {
  import_id: string;
  status: string;
  file_count: number;
  bytes_total: number;
  failures: number;
}

export interface StartImportArgs {
  card_label: string | null;
  card_mount: string | null;
  camera_model: string | null;
  dest_root: string;
  pattern: string;
  collision: 'skip' | 'rename' | 'overwrite';
  worker_count: number | null;
}

// ---------- Commands ----------

export const api = {
  listCards: () => invoke<Card[]>('list_cards'),
  scanCard: (mount: string) => invoke<ScanReport>('scan_card', { mount }),
  toolStatus: () => invoke<ToolStatus>('tool_status'),
  renderTemplatePreview: (pattern: string, files: FileInfo[], count = 5) =>
    invoke<string[]>('render_template_preview', { args: { pattern, files, count } }),
  startImport: (args: StartImportArgs, report: ScanReport) =>
    invoke<string>('start_import', { payload: { args, report } }),
  cancelImport: (importId: string) =>
    invoke<void>('cancel_import', { importId }),
  listImports: (limit = 100) => invoke<ImportRecord[]>('list_imports', { limit }),
  getImportFiles: (importId: string) =>
    invoke<ImportFileRow[]>('get_import_files', { importId }),
  getImportEvents: (importId: string) =>
    invoke<ImportEventRow[]>('get_import_events', { importId }),
  revealInFinder: (path: string) => invoke<void>('reveal_in_finder', { path }),
  ejectCard: (mount: string) => invoke<void>('eject_card', { mount }),
  pickDestinationDir: () => invoke<string | null>('pick_destination_dir'),
  getSettings: () => invoke<Settings>('get_settings'),
  setSettings: (settings: Settings) => invoke<void>('set_settings', { settings }),
  listTemplates: () => invoke<TemplateRow[]>('list_templates'),
  saveTemplate: (template: TemplateRow) => invoke<void>('save_template', { template }),
  deleteTemplate: (id: string) => invoke<void>('delete_template', { id }),
  getThumbnail: async (src: string): Promise<string> => {
    const path = await invoke<string>('get_thumbnail', { src });
    return convertFileSrc(path);
  },
};

// ---------- Events ----------

export function onCardAttached(cb: (card: Card) => void) {
  return listen<Card>('card-attached', (e: TauriEvent<Card>) => cb(e.payload));
}

export function onCardDetached(cb: (card: Card) => void) {
  return listen<Card>('card-detached', (e: TauriEvent<Card>) => cb(e.payload));
}

export function onImportProgress(cb: (p: ImportProgress) => void) {
  return listen<ImportProgress>('import-progress', (e) => cb(e.payload));
}

export function onImportComplete(cb: (p: ImportCompleted) => void) {
  return listen<ImportCompleted>('import-complete', (e) => cb(e.payload));
}
