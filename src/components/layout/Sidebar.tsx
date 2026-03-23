export type AppView = 'archive' | 'notes' | 'sync' | 'settings';

type SidebarProps = {
  activeView?: AppView;
  onNavigate?: (view: AppView) => void;
};

export default function Sidebar({ activeView = 'archive', onNavigate }: SidebarProps) {
  return (
    <aside className="sidebar" aria-label="Primary navigation">
      <button type="button" aria-pressed={activeView === 'archive'} onClick={() => onNavigate?.('archive')}>
        对话记录
      </button>
      <button type="button" aria-pressed={activeView === 'notes'} onClick={() => onNavigate?.('notes')}>
        知识笔记
      </button>
      <button type="button" aria-pressed={activeView === 'sync'} onClick={() => onNavigate?.('sync')}>
        同步日志
      </button>
      <button type="button" aria-pressed={activeView === 'settings'} onClick={() => onNavigate?.('settings')}>
        设置
      </button>
    </aside>
  );
}
