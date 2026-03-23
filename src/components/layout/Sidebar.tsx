export type AppView = 'archive' | 'notes' | 'sync' | 'settings';

type SidebarProps = {
  activeView?: AppView;
  onNavigate?: (view: AppView) => void;
};

export default function Sidebar({ activeView = 'archive', onNavigate }: SidebarProps) {
  return (
    <aside className="sidebar" aria-label="Primary navigation">
      <div className="sidebar__brand">
        <div className="sidebar__brand-mark">CV</div>
        <div>
          <strong>Context Vault</strong>
          <p>本地对话知识库</p>
        </div>
      </div>
      <div className="sidebar__section">
        <p className="sidebar__label">Workspace</p>
        <button
          type="button"
          className="sidebar__nav-item"
          data-active={activeView === 'archive'}
          aria-pressed={activeView === 'archive'}
          onClick={() => onNavigate?.('archive')}
        >
          对话记录
        </button>
        <button
          type="button"
          className="sidebar__nav-item"
          data-active={activeView === 'notes'}
          aria-pressed={activeView === 'notes'}
          onClick={() => onNavigate?.('notes')}
        >
          知识笔记
        </button>
        <button
          type="button"
          className="sidebar__nav-item"
          data-active={activeView === 'sync'}
          aria-pressed={activeView === 'sync'}
          onClick={() => onNavigate?.('sync')}
        >
          同步日志
        </button>
        <button
          type="button"
          className="sidebar__nav-item"
          data-active={activeView === 'settings'}
          aria-pressed={activeView === 'settings'}
          onClick={() => onNavigate?.('settings')}
        >
          设置
        </button>
      </div>
    </aside>
  );
}
