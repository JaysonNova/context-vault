import { invoke } from '@tauri-apps/api/core';
import type { ConversationListItem } from './types';

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
