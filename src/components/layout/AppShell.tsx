import Sidebar from './Sidebar';
import TopBar from './TopBar';

export default function AppShell() {
  return (
    <div className="app-shell">
      <Sidebar />
      <main className="content" />
      <TopBar />
    </div>
  );
}
