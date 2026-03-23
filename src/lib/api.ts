import { invoke } from '@tauri-apps/api/core';
import type { ConversationListItem, NoteListItem, SyncRunListItem } from './types';

type RawConversationListItem = {
  id: string;
  title: string;
  source_app: ConversationListItem['sourceApp'];
  sync_strength: ConversationListItem['syncStrength'];
  workspace_name: string | null;
  updated_at: number;
  note_count: number;
};

export async function listConversations(): Promise<ConversationListItem[]> {
  const rows = await invoke<RawConversationListItem[]>('list_conversations_command');

  return rows.map((row) => ({
    id: row.id,
    title: row.title,
    sourceApp: row.source_app,
    syncStrength: row.sync_strength,
    workspaceName: row.workspace_name ?? undefined,
    updatedAt: row.updated_at,
    noteCount: row.note_count
  }));
}

type RawSyncRunListItem = {
  source_app: string;
  status: string;
  imported_conversation_count: number;
  error_summary?: string | null;
};

export async function listSyncRuns(): Promise<SyncRunListItem[]> {
  const rows = await invoke<RawSyncRunListItem[]>('list_sync_runs_command');

  return rows.map((row) => ({
    sourceApp: row.source_app,
    status: row.status,
    importedConversationCount: row.imported_conversation_count,
    errorSummary: row.error_summary ?? undefined
  }));
}

export async function runSync(request: { trigger: string }): Promise<void> {
  await invoke('run_sync_command', { request });
}

export async function listNotes(): Promise<NoteListItem[]> {
  return invoke<NoteListItem[]>('list_notes_command');
}
