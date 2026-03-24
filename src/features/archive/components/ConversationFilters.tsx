import { useRef, useState, type PointerEvent as ReactPointerEvent } from 'react';
import type { ConversationListItem } from '../../../lib/types';

const SYNC_STRENGTH_OPTIONS: ConversationListItem['syncStrength'][] = [
  'partial',
  'full',
  'metadata_only'
];

type ConversationFiltersProps = {
  searchText: string;
  selectedSyncStrength?: string;
  resultCount: number;
  onSearchTextChange: (value: string) => void;
  onSearchSubmit: () => void;
  onSyncStrengthChange: (syncStrength?: string) => void;
};

export default function ConversationFilters({
  searchText,
  selectedSyncStrength,
  resultCount,
  onSearchTextChange,
  onSearchSubmit,
  onSyncStrengthChange
}: ConversationFiltersProps) {
  const chipsRef = useRef<HTMLDivElement>(null);
  const dragStateRef = useRef({
    pointerId: -1,
    startX: 0,
    scrollLeft: 0
  });
  const [isDragging, setIsDragging] = useState(false);

  const handlePointerDown = (event: ReactPointerEvent<HTMLDivElement>) => {
    const container = chipsRef.current;
    if (!container || container.scrollWidth <= container.clientWidth) {
      return;
    }

    dragStateRef.current = {
      pointerId: event.pointerId,
      startX: event.clientX,
      scrollLeft: container.scrollLeft
    };
    setIsDragging(true);
    if (typeof container.setPointerCapture === 'function') {
      container.setPointerCapture(event.pointerId);
    }
  };

  const handlePointerMove = (event: ReactPointerEvent<HTMLDivElement>) => {
    const container = chipsRef.current;
    if (!container || !isDragging || dragStateRef.current.pointerId !== event.pointerId) {
      return;
    }

    container.scrollLeft =
      dragStateRef.current.scrollLeft - (event.clientX - dragStateRef.current.startX);
  };

  const handlePointerUp = (event: ReactPointerEvent<HTMLDivElement>) => {
    const container = chipsRef.current;
    if (
      container &&
      typeof container.hasPointerCapture === 'function' &&
      container.hasPointerCapture(event.pointerId)
    ) {
      container.releasePointerCapture(event.pointerId);
    }

    dragStateRef.current.pointerId = -1;
    setIsDragging(false);
  };

  return (
    <section className="conversation-filters">
      <form
        className="conversation-filters__search"
        onSubmit={(event) => {
          event.preventDefault();
          onSearchSubmit();
        }}
      >
        <input
          type="search"
          value={searchText}
          placeholder="搜索标题、项目、关键词"
          onChange={(event) => onSearchTextChange(event.target.value)}
        />
        <button type="submit">搜索</button>
      </form>
      <div className="conversation-filters__summary">{resultCount} 条结果</div>
      <div
        ref={chipsRef}
        className="conversation-filters__chips"
        data-dragging={isDragging}
        onPointerDown={handlePointerDown}
        onPointerMove={handlePointerMove}
        onPointerUp={handlePointerUp}
        onPointerCancel={handlePointerUp}
      >
        <button
          type="button"
          aria-pressed={selectedSyncStrength === undefined}
          onClick={() => onSyncStrengthChange(undefined)}
        >
          全部
        </button>
        {SYNC_STRENGTH_OPTIONS.map((syncStrength) => (
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
