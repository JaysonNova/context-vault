import { useEffect, useState } from 'react';
import ArchivePagination from '../components/ArchivePagination';
import ArchiveSourceRail from '../components/ArchiveSourceRail';
import ConversationDetailPane from '../components/ConversationDetailPane';
import ConversationFilters from '../components/ConversationFilters';
import ConversationList from '../components/ConversationList';
import {
  getArchiveFacets,
  getConversationDetail,
  listConversations
} from '../../../lib/api';
import type {
  ArchiveFacets,
  ConversationDetail,
  ConversationListItem
} from '../../../lib/types';

type ArchivePageProps = {
  refreshKey?: number;
};

const DEFAULT_PAGE_SIZE = 10;

export default function ArchivePage({ refreshKey = 0 }: ArchivePageProps) {
  const [facets, setFacets] = useState<ArchiveFacets>({ totalCount: 0, sourceCounts: [] });
  const [conversations, setConversations] = useState<ConversationListItem[]>([]);
  const [searchTextDraft, setSearchTextDraft] = useState('');
  const [searchText, setSearchText] = useState('');
  const [isDetailExpanded, setIsDetailExpanded] = useState(false);
  const [pageSize, setPageSize] = useState(DEFAULT_PAGE_SIZE);
  const [currentPage, setCurrentPage] = useState(1);
  const [selectedSourceApp, setSelectedSourceApp] =
    useState<ConversationListItem['sourceApp']>();
  const [selectedSyncStrength, setSelectedSyncStrength] = useState<string>();
  const [selectedConversationId, setSelectedConversationId] = useState<string>();
  const [selectedConversationDetail, setSelectedConversationDetail] =
    useState<ConversationDetail | null>(null);

  useEffect(() => {
    void getArchiveFacets().then((items) => {
      setFacets(items);
    });
  }, [refreshKey]);

  useEffect(() => {
    let isMounted = true;

    void listConversations({
      text: searchText || undefined,
      sourceApp: selectedSourceApp,
      syncStrength: selectedSyncStrength as ConversationListItem['syncStrength'] | undefined
    }).then((items) => {
      if (!isMounted) {
        return;
      }

      setConversations(items);
      setCurrentPage(1);
    });

    return () => {
      isMounted = false;
    };
  }, [refreshKey, searchText, selectedSourceApp, selectedSyncStrength]);

  useEffect(() => {
    if (!selectedConversationId) {
      setSelectedConversationDetail(null);
      return;
    }

    let isMounted = true;
    void getConversationDetail(selectedConversationId).then((detail) => {
      if (isMounted) {
        setSelectedConversationDetail(detail);
      }
    });

    return () => {
      isMounted = false;
    };
  }, [selectedConversationId]);

  const totalPages = Math.max(1, Math.ceil(conversations.length / pageSize));
  const visiblePage = Math.min(currentPage, totalPages);
  const startIndex = (visiblePage - 1) * pageSize;
  const pagedConversations = conversations.slice(startIndex, startIndex + pageSize);
  const visibleStart = conversations.length === 0 ? 0 : startIndex + 1;
  const visibleEnd = startIndex + pagedConversations.length;

  useEffect(() => {
    if (currentPage !== visiblePage) {
      setCurrentPage(visiblePage);
    }
  }, [currentPage, visiblePage]);

  useEffect(() => {
    if (pagedConversations.length === 0) {
      setSelectedConversationId(undefined);
      return;
    }

    if (!selectedConversationId || !pagedConversations.some((item) => item.id === selectedConversationId)) {
      setSelectedConversationId(pagedConversations[0].id);
    }
  }, [pagedConversations, selectedConversationId]);

  const handleSearchTextChange = (value: string) => {
    setSearchTextDraft(value);
  };

  const handleSearchSubmit = () => {
    setCurrentPage(1);
    setSearchText(searchTextDraft);
  };

  const handleSourceAppChange = (sourceApp?: ConversationListItem['sourceApp']) => {
    setCurrentPage(1);
    setSelectedSourceApp(sourceApp);
  };

  const handleSyncStrengthChange = (syncStrength?: string) => {
    setCurrentPage(1);
    setSelectedSyncStrength(syncStrength);
  };

  const handlePageSizeChange = (nextPageSize: number) => {
    setCurrentPage(1);
    setPageSize(nextPageSize);
  };

  if (facets.totalCount === 0 && conversations.length === 0) {
    return <section>还没有同步任何对话</section>;
  }

  return (
    <section className="archive-page" data-detail-expanded={isDetailExpanded}>
      {!isDetailExpanded ? (
        <ArchiveSourceRail
          totalCount={facets.totalCount}
          sourceCounts={facets.sourceCounts}
          selectedSourceApp={selectedSourceApp}
          onSelectSource={handleSourceAppChange}
        />
      ) : null}
      {!isDetailExpanded ? (
        <div className="archive-page__main">
          <ConversationFilters
            searchText={searchTextDraft}
            selectedSyncStrength={selectedSyncStrength}
            resultCount={conversations.length}
            onSearchTextChange={handleSearchTextChange}
            onSearchSubmit={handleSearchSubmit}
            onSyncStrengthChange={handleSyncStrengthChange}
          />
          <ConversationList
            conversations={pagedConversations}
            selectedConversationId={selectedConversationId}
            onSelect={(conversation) => setSelectedConversationId(conversation.id)}
          />
          <ArchivePagination
            currentPage={visiblePage}
            totalPages={totalPages}
            pageSize={pageSize}
            totalCount={conversations.length}
            visibleStart={visibleStart}
            visibleEnd={visibleEnd}
            onPageChange={setCurrentPage}
            onPageSizeChange={handlePageSizeChange}
          />
        </div>
      ) : null}
      <ConversationDetailPane
        conversation={selectedConversationDetail}
        isExpanded={isDetailExpanded}
        onToggleExpanded={() => setIsDetailExpanded((current) => !current)}
      />
    </section>
  );
}
