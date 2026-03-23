import type { ReactNode } from 'react';
import Sidebar from './Sidebar';
import TopBar from './TopBar';
import type { AppView } from './Sidebar';

type AppShellProps = {
  activeView?: AppView;
  onNavigate?: (view: AppView) => void;
  onSync?: () => void;
  syncLabel?: string;
  children?: ReactNode;
};

export default function AppShell({
  activeView = 'archive',
  onNavigate,
  onSync,
  syncLabel,
  children
}: AppShellProps) {
  return (
    <div className="app-shell">
      <Sidebar activeView={activeView} onNavigate={onNavigate} />
      <section className="content-shell">
        <TopBar activeView={activeView} onSync={onSync} syncLabel={syncLabel} />
        <main className="content">{children}</main>
      </section>
    </div>
  );
}
