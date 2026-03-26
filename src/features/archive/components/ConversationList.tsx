import type { ConversationListItem } from '../../../lib/types';
import { formatSourceLabel, formatTimestamp } from '../../../lib/format';

type ConversationListProps = {
  conversations: ConversationListItem[];
  selectedConversationId?: string;
  onSelect: (conversation: ConversationListItem) => void;
  onDeleteRequest?: (conversation: ConversationListItem) => void;
};

export default function ConversationList({
  conversations,
  selectedConversationId,
  onSelect,
  onDeleteRequest
}: ConversationListProps) {
  if (conversations.length === 0) {
    return <div className="conversation-list conversation-list--empty">当前筛选条件下暂无对话</div>;
  }

  return (
    <div className="conversation-list">
      {conversations.map((conversation) => {
        const canDelete =
          onDeleteRequest &&
          (conversation.sourceApp === 'codex' || conversation.sourceApp === 'claude_code');

        return (
          <div key={conversation.id} className="conversation-list__item">
            <button
              type="button"
              aria-label={conversation.title}
              title={conversation.title}
              className={`conversation-list__row${canDelete ? ' conversation-list__row--deletable' : ''}`}
              data-selected={conversation.id === selectedConversationId}
              data-source={conversation.sourceApp}
              data-strength={conversation.syncStrength}
              onClick={() => onSelect(conversation)}
            >
              <div className="conversation-list__title-row">
                <strong className="conversation-list__title">{conversation.title}</strong>
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
            {canDelete ? (
              <button
                type="button"
                className="conversation-list__delete"
                aria-label={`删除 ${conversation.title}`}
                onClick={(event) => {
                  event.stopPropagation();
                  onDeleteRequest(conversation);
                }}
              >
                删除
              </button>
            ) : null}
          </div>
        );
      })}
    </div>
  );
}
