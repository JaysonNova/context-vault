const PAGE_SIZE_OPTIONS = [10, 20, 50] as const;

type ArchivePaginationProps = {
  currentPage: number;
  totalPages: number;
  pageSize: number;
  totalCount: number;
  visibleStart: number;
  visibleEnd: number;
  onPageChange: (page: number) => void;
  onPageSizeChange: (pageSize: number) => void;
};

export default function ArchivePagination({
  currentPage,
  totalPages,
  pageSize,
  totalCount,
  visibleStart,
  visibleEnd,
  onPageChange,
  onPageSizeChange
}: ArchivePaginationProps) {
  return (
    <section className="archive-pagination" aria-label="分页导航">
      <div className="archive-pagination__summary">
        当前显示 {visibleStart}-{visibleEnd} / {totalCount} 条
      </div>
      <div className="archive-pagination__controls">
        <label className="archive-pagination__size">
          <span>每页</span>
          <select
            aria-label="每页条数"
            value={pageSize}
            onChange={(event) => onPageSizeChange(Number(event.target.value))}
          >
            {PAGE_SIZE_OPTIONS.map((option) => (
              <option key={option} value={option}>
                {option}
              </option>
            ))}
          </select>
        </label>
        <button
          type="button"
          disabled={currentPage <= 1}
          onClick={() => onPageChange(currentPage - 1)}
        >
          上一页
        </button>
        <span className="archive-pagination__status">
          第 {currentPage} / {totalPages} 页
        </span>
        <button
          type="button"
          disabled={currentPage >= totalPages}
          onClick={() => onPageChange(currentPage + 1)}
        >
          下一页
        </button>
      </div>
    </section>
  );
}
