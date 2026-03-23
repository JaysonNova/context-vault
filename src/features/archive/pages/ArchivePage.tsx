import { useEffect, useState } from 'react';
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

export default function ArchivePage({ refreshKey = 0 }: ArchivePageProps) {
  const [facets, setFacets] = useState<ArchiveFacets>({ totalCount: 0, sourceCounts: [] });
  const [conversations, setConversations] = useState<ConversationListItem[]>([]);
  const [searchText, setSearchText] = useState('');
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
      setSelectedConversationId((current) => {
        if (current && items.some((item) => item.id === current)) {
          return current;
        }
        return items[0]?.id;
      });
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

  const syncStrengths = Array.from(
    new Set(conversations.map((conversation) => conversation.syncStrength))
  );

  const handleSyncStrengthChange = (syncStrength?: string) => {
    setSelectedSyncStrength(syncStrength);
  };

  if (conversations.length === 0) {
    return <section>还没有同步任何对话</section>;
  }

  return (
    <section className="archive-page">
      <ArchiveSourceRail
        totalCount={facets.totalCount}
        sourceCounts={facets.sourceCounts}
        selectedSourceApp={selectedSourceApp}
        onSelectSource={setSelectedSourceApp}
      />
      <div className="archive-page__main">
        <ConversationFilters
          searchText={searchText}
          selectedSyncStrength={selectedSyncStrength}
          syncStrengths={syncStrengths}
          resultCount={conversations.length}
          onSearchTextChange={setSearchText}
          onSyncStrengthChange={handleSyncStrengthChange}
        />
        <ConversationList
          conversations={conversations}
          selectedConversationId={selectedConversationId}
          onSelect={(conversation) => setSelectedConversationId(conversation.id)}
        />
      </div>
      <ConversationDetailPane conversation={selectedConversationDetail} />
    </section>
  );
}
