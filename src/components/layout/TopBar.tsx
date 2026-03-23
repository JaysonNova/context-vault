type TopBarProps = {
  activeView?: string;
  onSync?: () => void;
  syncLabel?: string;
};

const viewTitles: Record<string, { title: string; subtitle: string }> = {
  archive: { title: '对话记录', subtitle: '统一归档本地 AI 编程对话' },
  notes: { title: '知识笔记', subtitle: '沉淀后的可搜索技术笔记' },
  sync: { title: '同步日志', subtitle: '查看最近的导入与异常记录' },
  settings: { title: '设置', subtitle: '本地应用与模型接入配置' }
};

export default function TopBar({
  activeView = 'archive',
  onSync,
  syncLabel = '同步'
}: TopBarProps) {
  const currentCopy = viewTitles[activeView] ?? viewTitles.archive;

  return (
    <header className="topbar">
      <div className="topbar__copy">
        <p>{currentCopy.subtitle}</p>
        <h1>{currentCopy.title}</h1>
      </div>
      <div className="topbar__actions">
        <span className="topbar__hint">启动扫描 + 手动同步</span>
        <button type="button" onClick={onSync}>
          {syncLabel}
        </button>
      </div>
    </header>
  );
}
