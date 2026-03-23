import { useEffect, useState } from 'react';
import ConversationDetailPane from '../components/ConversationDetailPane';
import ConversationList from '../components/ConversationList';
import { listConversations } from '../../../lib/api';
import type { ConversationListItem } from '../../../lib/types';

export default function ArchivePage() {
  const [conversations, setConversations] = useState<ConversationListItem[]>([]);
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
  }, []);

  if (conversations.length === 0) {
    return <section>还没有同步任何对话</section>;
  }

  return (
    <section className="archive-page">
      <ConversationList
        conversations={conversations}
        selectedConversationId={selectedConversation?.id}
        onSelect={setSelectedConversation}
      />
      <ConversationDetailPane conversation={selectedConversation} />
    </section>
  );
}
