import type { ConversationListItem } from '../../../lib/types';
import { formatSourceLabel, formatTimestamp } from '../../../lib/format';

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
          aria-label={conversation.title}
          className="conversation-list__row"
          data-selected={conversation.id === selectedConversationId}
          data-source={conversation.sourceApp}
          data-strength={conversation.syncStrength}
          onClick={() => onSelect(conversation)}
        >
          <div className="conversation-list__title-row">
            <strong>{conversation.title}</strong>
            <span className="conversation-list__strength">{conversation.syncStrength}</span>
          </div>
          <p className="conversation-list__preview">{conversation.previewText || '暂无摘要预览'}</p>
          <div className="conversation-list__meta">
            <span>{formatSourceLabel(conversation.sourceApp)}</span>
            <span>{conversation.workspaceName ?? '未识别项目'}</span>
            <span>{conversation.messageCount} 条消息</span>
            <span>{conversation.noteCount} 条笔记</span>
            <span>{formatTimestamp(conversation.updatedAt)}</span>
          </div>
        </button>
      ))}
    </div>
  );
}
