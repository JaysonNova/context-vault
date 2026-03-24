import { startTransition, useEffect, useState, type ReactNode } from 'react';
import AppShell from './components/layout/AppShell';
import type { AppView } from './components/layout/Sidebar';
import ArchivePage from './features/archive/pages/ArchivePage';
import NotesPage from './features/notes/pages/NotesPage';
import SettingsPage from './features/settings/pages/SettingsPage';
import SyncLogPage from './features/sync/pages/SyncLogPage';
import { runSync } from './lib/api';

export default function App() {
  const [activeView, setActiveView] = useState<AppView>('archive');
  const [refreshKey, setRefreshKey] = useState(0);
  const [isSyncing, setIsSyncing] = useState(false);
  const [visitedViews, setVisitedViews] = useState<Record<AppView, boolean>>({
    archive: true,
    notes: false,
    sync: false,
    settings: false
  });

  const handleSync = async (trigger: string) => {
    if (isSyncing) {
      return;
    }

    setIsSyncing(true);
    try {
      await runSync({ trigger });
      setRefreshKey((current) => current + 1);
    } finally {
      setIsSyncing(false);
    }
  };

  useEffect(() => {
    void handleSync('startup');
  }, []);

  useEffect(() => {
    setVisitedViews((current) => {
      if (current[activeView]) {
        return current;
      }

      return {
        ...current,
        [activeView]: true
      };
    });
  }, [activeView]);

  const handleNavigate = (view: AppView) => {
    startTransition(() => {
      setActiveView(view);
    });
  };

  const renderView = (view: AppView, content: ReactNode) => {
    if (!visitedViews[view]) {
      return null;
    }

    return (
      <section
        className="app-view"
        data-view={view}
        hidden={activeView !== view}
        aria-hidden={activeView !== view}
      >
        {content}
      </section>
    );
  };

  return (
    <AppShell
      activeView={activeView}
      onNavigate={handleNavigate}
      onSync={() => void handleSync('manual')}
      isSyncing={isSyncing}
    >
      {renderView('archive', <ArchivePage refreshKey={refreshKey} />)}
      {renderView('notes', <NotesPage refreshKey={refreshKey} />)}
      {renderView('sync', <SyncLogPage refreshKey={refreshKey} />)}
      {renderView('settings', <SettingsPage />)}
    </AppShell>
  );
}
