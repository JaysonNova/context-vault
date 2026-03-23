import { useEffect } from 'react';
import AppShell from './components/layout/AppShell';
import { runSync } from './lib/api';

export default function App() {
  useEffect(() => {
    void runSync({ trigger: 'startup' });
  }, []);

  return <AppShell />;
}
