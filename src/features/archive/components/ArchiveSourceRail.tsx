import type { ArchiveSourceCount, ConversationListItem } from '../../../lib/types';
import { formatSourceLabel } from '../../../lib/format';

type ArchiveSourceRailProps = {
  totalCount: number;
  sourceCounts: ArchiveSourceCount[];
  selectedSourceApp?: ConversationListItem['sourceApp'];
  onSelectSource: (sourceApp?: ConversationListItem['sourceApp']) => void;
};

export default function ArchiveSourceRail({
  totalCount,
  sourceCounts,
  selectedSourceApp,
  onSelectSource
}: ArchiveSourceRailProps) {
  return (
    <aside className="archive-source-rail">
      <div className="archive-source-rail__section">
        <p className="archive-source-rail__label">来源</p>
        <button
          type="button"
          className="archive-source-rail__item"
          data-active={selectedSourceApp === undefined}
          onClick={() => onSelectSource(undefined)}
        >
          <span>全部对话</span>
          <strong>{totalCount}</strong>
        </button>
        {sourceCounts.map((item) => (
          <button
            key={item.sourceApp}
            type="button"
            className="archive-source-rail__item"
            data-active={selectedSourceApp === item.sourceApp}
            onClick={() => onSelectSource(item.sourceApp)}
          >
            <span>{formatSourceLabel(item.sourceApp)}</span>
            <strong>{item.count}</strong>
          </button>
        ))}
      </div>
    </aside>
  );
}
