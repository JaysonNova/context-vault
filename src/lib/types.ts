export type ConversationListItem = {
  id: string;
  title: string;
  sourceApp: 'codex' | 'claude_code' | 'cursor';
  syncStrength: 'full' | 'partial' | 'metadata_only';
  workspaceName?: string;
  updatedAt: number;
  messageCount: number;
  previewText: string;
  noteCount: number;
};

export type ArchiveQuery = {
  text?: string;
  sourceApp?: ConversationListItem['sourceApp'];
  syncStrength?: ConversationListItem['syncStrength'];
  workspaceId?: string;
  noteState?: 'with_notes' | 'without_notes';
};

export type ArchiveSourceCount = {
  sourceApp: ConversationListItem['sourceApp'];
  count: number;
};

export type ArchiveFacets = {
  totalCount: number;
  sourceCounts: ArchiveSourceCount[];
};

export type ConversationDetailMessage = {
  id: string;
  role: string;
  messageType: string;
  contentText: string;
  createdAt: number;
};

export type ConversationDetail = {
  id: string;
  title: string;
  sourceApp: ConversationListItem['sourceApp'];
  syncStrength: ConversationListItem['syncStrength'];
  workspaceName?: string;
  updatedAt: number;
  noteCount: number;
  previewText: string;
  rawMetadataJson: string;
  messages: ConversationDetailMessage[];
};

export type SyncRunListItem = {
  sourceApp: string;
  status: string;
  importedConversationCount: number;
  errorSummary?: string;
};

export type NoteListItem = {
  id: string;
  title: string;
  summary: string;
};
