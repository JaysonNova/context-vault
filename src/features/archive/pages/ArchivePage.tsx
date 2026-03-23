import { useEffect, useState } from 'react';
import ConversationDetailPane from '../components/ConversationDetailPane';
import ConversationFilters from '../components/ConversationFilters';
import ConversationList from '../components/ConversationList';
import { listConversations } from '../../../lib/api';
import type { ConversationListItem } from '../../../lib/types';

type ArchivePageProps = {
  refreshKey?: number;
};

export default function ArchivePage({ refreshKey = 0 }: ArchivePageProps) {
  const [conversations, setConversations] = useState<ConversationListItem[]>([]);
  const [selectedSyncStrength, setSelectedSyncStrength] = useState<string>();
  const [selectedConversation, setSelectedConversation] = useState<
    ConversationListItem | undefined
  >();

  useEffect(() => {
    let isMounted = true;

    void listConversations().then((items) => {
      if (!isMounted) {
        return;
      }

      setConversations(items);
      setSelectedConversation(items[0]);
    });

    return () => {
      isMounted = false;
    };
  }, [refreshKey]);

  const filteredConversations = conversations.filter((conversation) =>
    selectedSyncStrength ? conversation.syncStrength === selectedSyncStrength : true
  );

  const syncStrengths = Array.from(new Set(conversations.map((conversation) => conversation.syncStrength)));

  const handleSyncStrengthChange = (syncStrength?: string) => {
    setSelectedSyncStrength(syncStrength);
    const nextConversation = conversations.find((conversation) =>
      syncStrength ? conversation.syncStrength === syncStrength : true
    );
    setSelectedConversation(nextConversation);
  };

  if (conversations.length === 0) {
    return <section>还没有同步任何对话</section>;
  }

  return (
    <section className="archive-page">
      <ConversationFilters
        selectedSyncStrength={selectedSyncStrength}
        syncStrengths={syncStrengths}
        onSyncStrengthChange={handleSyncStrengthChange}
      />
      <ConversationList
        conversations={filteredConversations}
        selectedConversationId={selectedConversation?.id}
        onSelect={setSelectedConversation}
      />
      <ConversationDetailPane conversation={selectedConversation} />
    </section>
  );
}
