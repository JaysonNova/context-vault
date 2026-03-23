import type { ConversationListItem } from '../../../lib/types';

type ConversationListProps = {
  conversations: ConversationListItem[];
  selectedConversationId?: string;
  onSelect: (conversation: ConversationListItem) => void;
};

export default function ConversationList({
  conversations,
  selectedConversationId,
  onSelect
}: ConversationListProps) {
  return (
    <div className="conversation-list">
      {conversations.map((conversation) => (
        <button
          key={conversation.id}
          type="button"
          className={conversation.id === selectedConversationId ? 'is-selected' : ''}
          onClick={() => onSelect(conversation)}
        >
          {conversation.title}
        </button>
      ))}
    </div>
  );
}
