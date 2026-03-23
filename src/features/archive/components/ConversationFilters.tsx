type ConversationFiltersProps = {
  selectedSyncStrength?: string;
  syncStrengths: string[];
  onSyncStrengthChange: (syncStrength?: string) => void;
};

export default function ConversationFilters({
  selectedSyncStrength,
  syncStrengths,
  onSyncStrengthChange
}: ConversationFiltersProps) {
  return (
    <div className="conversation-filters">
      <button
        type="button"
        aria-pressed={selectedSyncStrength === undefined}
        onClick={() => onSyncStrengthChange(undefined)}
      >
        全部
      </button>
      {syncStrengths.map((syncStrength) => (
        <button
          key={syncStrength}
          type="button"
          aria-pressed={selectedSyncStrength === syncStrength}
          onClick={() => onSyncStrengthChange(syncStrength)}
        >
          {syncStrength}
        </button>
      ))}
    </div>
  );
}
