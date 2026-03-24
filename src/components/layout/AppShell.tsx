import type { ReactNode } from 'react';
import Sidebar from './Sidebar';
import TopBar from './TopBar';
import type { AppView } from './Sidebar';

type AppShellProps = {
  activeView?: AppView;
  onNavigate?: (view: AppView) => void;
  onSync?: () => void;
  isSyncing?: boolean;
  children?: ReactNode;
};

export default function AppShell({
  activeView = 'archive',
  onNavigate,
  onSync,
  isSyncing,
  children
}: AppShellProps) {
  return (
    <div className="app-shell">
      <Sidebar activeView={activeView} onNavigate={onNavigate} />
      <section className="content-shell">
        <TopBar activeView={activeView} onSync={onSync} isSyncing={isSyncing} />
        <main
          className="content"
          data-scroll-mode={activeView === 'archive' ? 'contained' : 'page'}
        >
          {children}
        </main>
      </section>
    </div>
  );
}
