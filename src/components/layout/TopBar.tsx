type TopBarProps = {
  onSync?: () => void;
  syncLabel?: string;
};

export default function TopBar({ onSync, syncLabel = '同步' }: TopBarProps) {
  return (
    <header className="topbar">
      <button type="button" onClick={onSync}>
        {syncLabel}
      </button>
    </header>
  );
}
