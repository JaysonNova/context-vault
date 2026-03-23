export type ConversationListItem = {
  id: string;
  title: string;
  sourceApp: 'codex' | 'claude_code' | 'cursor';
  syncStrength: 'full' | 'partial' | 'metadata_only';
  workspaceName?: string;
  updatedAt: number;
  noteCount: number;
};
