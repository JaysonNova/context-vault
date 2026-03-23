import { useEffect, useState } from 'react';
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
  const [syncLabel, setSyncLabel] = useState('同步');

  const handleSync = async (trigger: string) => {
    setSyncLabel('同步中...');
    try {
      await runSync({ trigger });
      setRefreshKey((current) => current + 1);
    } finally {
      setSyncLabel('同步');
    }
  };

  useEffect(() => {
    void handleSync('startup');
  }, []);

  return (
    <AppShell
      activeView={activeView}
      onNavigate={setActiveView}
      onSync={() => void handleSync('manual')}
      syncLabel={syncLabel}
    >
      {activeView === 'archive' ? <ArchivePage refreshKey={refreshKey} /> : null}
      {activeView === 'notes' ? <NotesPage refreshKey={refreshKey} /> : null}
      {activeView === 'sync' ? <SyncLogPage refreshKey={refreshKey} /> : null}
      {activeView === 'settings' ? <SettingsPage /> : null}
    </AppShell>
  );
}
