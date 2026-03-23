export function formatTimestamp(timestamp: number): string {
  return new Intl.DateTimeFormat('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  }).format(new Date(timestamp));
}

export function formatSourceLabel(sourceApp: string): string {
  switch (sourceApp) {
    case 'claude_code':
      return 'Claude Code';
    case 'codex':
      return 'CodeX';
    case 'cursor':
      return 'Cursor';
    default:
      return sourceApp;
  }
}
