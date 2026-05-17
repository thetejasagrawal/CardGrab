import { writable, derived } from 'svelte/store';
import type {
  Card,
  ScanReport,
  ImportProgress,
  ImportRecord,
  Settings,
  TemplateRow,
  ToolStatus,
} from './api';

export type View =
  | { kind: 'empty' }
  | { kind: 'card'; mount: string }
  | { kind: 'progress'; importId: string }
  | { kind: 'history' }
  | { kind: 'import-detail'; importId: string }
  | { kind: 'settings' };

export const cards = writable<Card[]>([]);
export const scanByMount = writable<Record<string, ScanReport | 'loading' | 'error'>>({});
export const currentView = writable<View>({ kind: 'empty' });
export const activeProgress = writable<ImportProgress | null>(null);
export const importHistory = writable<ImportRecord[]>([]);
export const settings = writable<Settings | null>(null);
export const templates = writable<TemplateRow[]>([]);
export const toolStatus = writable<ToolStatus | null>(null);
export const lastCompleted = writable<{
  importId: string;
  destRoot: string;
  status: string;
  fileCount: number;
  bytesTotal: number;
  failures: number;
} | null>(null);

export const firstCard = derived(cards, ($c) => $c[0] ?? null);
