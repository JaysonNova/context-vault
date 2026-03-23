import type { ConversationDetail } from '../../../lib/types';
import { formatSourceLabel, formatTimestamp } from '../../../lib/format';

type ConversationDetailPaneProps = {
  conversation?: ConversationDetail | null;
};

export default function ConversationDetailPane({
  conversation
}: ConversationDetailPaneProps) {
  const formattedMetadata = conversation
    ? (() => {
        try {
          return JSON.stringify(JSON.parse(conversation.rawMetadataJson), null, 2);
        } catch {
          return conversation.rawMetadataJson;
        }
      })()
    : '';

  if (!conversation) {
    return <section className="detail-pane detail-pane--empty">请选择一条对话查看详情</section>;
  }

  return (
    <section className="detail-pane">
      <div className="detail-pane__header">
        <div>
          <p className="detail-pane__eyebrow">{formatSourceLabel(conversation.sourceApp)}</p>
          <h2>{conversation.title}</h2>
        </div>
        <span className="detail-pane__badge">{conversation.syncStrength}</span>
      </div>
      <div className="detail-pane__meta">
        <span>{conversation.workspaceName ?? '未识别项目'}</span>
        <span>{formatTimestamp(conversation.updatedAt)}</span>
        <span>{conversation.noteCount} 条笔记</span>
      </div>
      <section className="detail-pane__section">
        <h3>摘要预览</h3>
        <p>{conversation.previewText || '暂无摘要预览'}</p>
      </section>
      <section className="detail-pane__section">
        <h3>消息时间线</h3>
        <div className="detail-pane__timeline">
          {conversation.messages.length === 0 ? (
            <p className="detail-pane__empty-copy">当前来源暂无可展示的消息时间线。</p>
          ) : (
            conversation.messages.map((message) => (
              <article key={message.id} className="detail-pane__message">
                <div className="detail-pane__message-meta">
                  <span>{message.role}</span>
                  <span>{message.messageType}</span>
                  <span>{formatTimestamp(message.createdAt)}</span>
                </div>
                <p>{message.contentText}</p>
              </article>
            ))
          )}
        </div>
      </section>
      <section className="detail-pane__section">
        <h3>原始元数据</h3>
        <pre>{formattedMetadata}</pre>
      </section>
    </section>
  );
}
