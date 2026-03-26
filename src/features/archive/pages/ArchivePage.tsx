import { memo, useEffect, useState } from 'react';
import ArchivePagination from '../components/ArchivePagination';
import ArchiveSourceRail from '../components/ArchiveSourceRail';
import ConversationDetailPane from '../components/ConversationDetailPane';
import ConversationFilters from '../components/ConversationFilters';
import ConversationList from '../components/ConversationList';
import {
  getArchiveFacets,
  getConversationDetail,
  listConversations,
  softDeleteConversation
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
const DELETE_DIALOG_TITLE_ID = 'delete-conversation-dialog-title';
const DELETE_DIALOG_DESCRIPTION_ID = 'delete-conversation-dialog-description';

function ArchivePage({ refreshKey = 0 }: ArchivePageProps) {
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
  const [pendingDeleteConversation, setPendingDeleteConversation] =
    useState<ConversationListItem | null>(null);
  const [deleteError, setDeleteError] = useState<string>();
  const [isDeletingConversation, setIsDeletingConversation] = useState(false);

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

  const handleDeleteRequest = (conversation: ConversationListItem) => {
    setDeleteError(undefined);
    setPendingDeleteConversation(conversation);
  };

  const handleDeleteCancel = () => {
    if (isDeletingConversation) {
      return;
    }

    setDeleteError(undefined);
    setPendingDeleteConversation(null);
  };

  const handleDeleteConfirm = async () => {
    if (!pendingDeleteConversation || isDeletingConversation) {
      return;
    }

    setIsDeletingConversation(true);
    setDeleteError(undefined);

    try {
      const deletedConversationId = pendingDeleteConversation.id;
      await softDeleteConversation(deletedConversationId);

      if (selectedConversationId === deletedConversationId) {
        setSelectedConversationId(undefined);
        setSelectedConversationDetail(null);
      }

      const [nextConversations, nextFacets] = await Promise.all([
        listConversations({
          text: searchText || undefined,
          sourceApp: selectedSourceApp,
          syncStrength: selectedSyncStrength as ConversationListItem['syncStrength'] | undefined
        }),
        getArchiveFacets()
      ]);

      setConversations(nextConversations);
      setFacets(nextFacets);
      setPendingDeleteConversation(null);
    } catch (error) {
      setDeleteError(getErrorMessage(error));
    } finally {
      setIsDeletingConversation(false);
    }
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
            onDeleteRequest={handleDeleteRequest}
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
      {pendingDeleteConversation ? (
        <div className="archive-dialog-backdrop">
          <section
            className="archive-dialog"
            role="dialog"
            aria-modal="true"
            aria-labelledby={DELETE_DIALOG_TITLE_ID}
            aria-describedby={DELETE_DIALOG_DESCRIPTION_ID}
          >
            <div className="archive-dialog__copy">
              <p className="archive-dialog__eyebrow">删除确认</p>
              <h2 id={DELETE_DIALOG_TITLE_ID}>确认删除历史记录</h2>
              <p id={DELETE_DIALOG_DESCRIPTION_ID}>
                仅从 Context Vault 已同步数据库中隐藏，不删除本地 Claude/Codex 原始记录。
              </p>
              <p className="archive-dialog__conversation">{pendingDeleteConversation.title}</p>
              {deleteError ? <p className="archive-dialog__error">{deleteError}</p> : null}
            </div>
            <div className="archive-dialog__actions">
              <button type="button" onClick={handleDeleteCancel} disabled={isDeletingConversation}>
                取消
              </button>
              <button
                type="button"
                className="archive-dialog__confirm"
                onClick={() => void handleDeleteConfirm()}
                disabled={isDeletingConversation}
              >
                {isDeletingConversation ? '删除中' : '删除'}
              </button>
            </div>
          </section>
        </div>
      ) : null}
    </section>
  );
}

ArchivePage.displayName = 'ArchivePage';

export default memo(ArchivePage);

function getErrorMessage(error: unknown) {
  if (typeof error === 'string' && error.trim()) {
    return error;
  }

  if (error instanceof Error && error.message.trim()) {
    return error.message;
  }

  return '删除失败，请稍后重试。';
}
