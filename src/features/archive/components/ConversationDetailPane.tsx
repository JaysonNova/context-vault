import type { ConversationListItem } from '../../../lib/types';
import { formatTimestamp } from '../../../lib/format';

type ConversationDetailPaneProps = {
  conversation?: ConversationListItem;
};

export default function ConversationDetailPane({
  conversation
}: ConversationDetailPaneProps) {
  if (!conversation) {
    return <section className="detail-pane">请选择一条对话查看详情</section>;
  }

  return (
    <section className="detail-pane">
      <h2>{conversation.title}</h2>
      <p>{conversation.syncStrength}</p>
      <p>{formatTimestamp(conversation.updatedAt)}</p>
    </section>
  );
}
