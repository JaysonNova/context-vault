import { invoke } from '@tauri-apps/api/core';
import type {
  ArchiveFacets,
  ArchiveQuery,
  ConversationDetail,
  ConversationListItem,
  NoteListItem,
  SyncRunListItem
} from './types';

type RawConversationListItem = {
  id: string;
  title: string;
  source_app: ConversationListItem['sourceApp'];
  sync_strength: ConversationListItem['syncStrength'];
  workspace_name: string | null;
  updated_at: number;
  message_count: number;
  preview_text: string;
  note_count: number;
};

export async function listConversations(query: ArchiveQuery = {}): Promise<ConversationListItem[]> {
  const rows = await invoke<RawConversationListItem[]>('list_conversations_command', {
    query: {
      text: query.text,
      source_app: query.sourceApp,
      sync_strength: query.syncStrength,
      workspace_id: query.workspaceId,
      note_state: query.noteState
    }
  });

  return rows.map((row) => ({
    id: row.id,
    title: row.title,
    sourceApp: row.source_app,
    syncStrength: row.sync_strength,
    workspaceName: row.workspace_name ?? undefined,
    updatedAt: row.updated_at,
    messageCount: row.message_count,
    previewText: row.preview_text,
    noteCount: row.note_count
  }));
}

type RawArchiveFacets = {
  total_count: number;
  source_counts: Array<{
    source_app: ConversationListItem['sourceApp'];
    count: number;
  }>;
};

export async function getArchiveFacets(): Promise<ArchiveFacets> {
  const facets = await invoke<RawArchiveFacets>('get_archive_facets_command');

  return {
    totalCount: facets.total_count,
    sourceCounts: facets.source_counts.map((item) => ({
      sourceApp: item.source_app,
      count: item.count
    }))
  };
}

type RawConversationDetail = {
  id: string;
  title: string;
  source_app: ConversationListItem['sourceApp'];
  sync_strength: ConversationListItem['syncStrength'];
  workspace_name: string | null;
  updated_at: number;
  note_count: number;
  preview_text: string;
  raw_metadata_json: string;
  messages: Array<{
    id: string;
    role: string;
    message_type: string;
    content_text: string;
    created_at: number;
  }>;
};

export async function getConversationDetail(
  conversationId: string
): Promise<ConversationDetail | null> {
  const detail = await invoke<RawConversationDetail | null>('get_conversation_detail_command', {
    conversationId
  });

  if (!detail) {
    return null;
  }

  return {
    id: detail.id,
    title: detail.title,
    sourceApp: detail.source_app,
    syncStrength: detail.sync_strength,
    workspaceName: detail.workspace_name ?? undefined,
    updatedAt: detail.updated_at,
    noteCount: detail.note_count,
    previewText: detail.preview_text,
    rawMetadataJson: detail.raw_metadata_json,
    messages: detail.messages.map((message) => ({
      id: message.id,
      role: message.role,
      messageType: message.message_type,
      contentText: message.content_text,
      createdAt: message.created_at
    }))
  };
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
