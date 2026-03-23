import type { ConversationListItem } from '../../../lib/types';

type ConversationFiltersProps = {
  searchText: string;
  selectedSyncStrength?: string;
  syncStrengths: ConversationListItem['syncStrength'][];
  resultCount: number;
  onSearchTextChange: (value: string) => void;
  onSyncStrengthChange: (syncStrength?: string) => void;
};

export default function ConversationFilters({
  searchText,
  selectedSyncStrength,
  syncStrengths,
  resultCount,
  onSearchTextChange,
  onSyncStrengthChange
}: ConversationFiltersProps) {
  return (
    <section className="conversation-filters">
      <div className="conversation-filters__search">
        <input
          type="search"
          value={searchText}
          placeholder="搜索标题、项目、关键词"
          onChange={(event) => onSearchTextChange(event.target.value)}
        />
      </div>
      <div className="conversation-filters__summary">{resultCount} 条结果</div>
      <div className="conversation-filters__chips">
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
    </section>
  );
}
