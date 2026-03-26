import { useEffect, useState } from 'react';
import type { ConversationDetail } from '../../../lib/types';
import { formatSourceLabel, formatTimestamp } from '../../../lib/format';

type ConversationDetailPaneProps = {
  conversation?: ConversationDetail | null;
  isExpanded?: boolean;
  onToggleExpanded?: () => void;
};

type ResumeCopyStatus = 'idle' | 'success' | 'error';

export default function ConversationDetailPane({
  conversation,
  isExpanded = false,
  onToggleExpanded
}: ConversationDetailPaneProps) {
  const [collapsedToolMessages, setCollapsedToolMessages] = useState<Record<string, boolean>>({});
  const [resumeCopyStatus, setResumeCopyStatus] = useState<ResumeCopyStatus>('idle');
  const formattedMetadata = conversation
    ? (() => {
        try {
          return JSON.stringify(JSON.parse(conversation.rawMetadataJson), null, 2);
        } catch {
          return conversation.rawMetadataJson;
        }
      })()
    : '';

  useEffect(() => {
    setCollapsedToolMessages({});
    setResumeCopyStatus('idle');
  }, [conversation?.id]);

  useEffect(() => {
    if (resumeCopyStatus === 'idle') {
      return;
    }

    const timeoutId = window.setTimeout(() => {
      setResumeCopyStatus('idle');
    }, 2000);

    return () => {
      window.clearTimeout(timeoutId);
    };
  }, [resumeCopyStatus]);

  if (!conversation) {
    return <section className="detail-pane detail-pane--empty">请选择一条对话查看详情</section>;
  }

  const sourceNotice = getSourceNotice(conversation.syncStrength);
  const showTimeline = conversation.syncStrength !== 'metadata_only';
  const resumeCopyLabel = getResumeCopyLabel(resumeCopyStatus);

  const handleCopyResumeCommand = () => {
    if (!conversation.resumeCommand) {
      return;
    }

    void copyText(conversation.resumeCommand)
      .then(() => {
        setResumeCopyStatus('success');
      })
      .catch(() => {
        setResumeCopyStatus('error');
      });
  };

  return (
    <section className="detail-pane" data-expanded={isExpanded}>
      <div className="detail-pane__sticky-actions">
        <span className="detail-pane__badge">{conversation.syncStrength}</span>
        {onToggleExpanded ? (
          <button
            type="button"
            className="detail-pane__toggle"
            data-expanded={isExpanded}
            onClick={onToggleExpanded}
          >
            {isExpanded ? '收起详情' : '展开详情'}
          </button>
        ) : null}
        {conversation.resumeCommand ? (
          <button
            type="button"
            className="detail-pane__resume-copy"
            data-status={resumeCopyStatus}
            aria-live="polite"
            onClick={handleCopyResumeCommand}
          >
            {resumeCopyLabel}
          </button>
        ) : null}
      </div>
      <div className="detail-pane__header">
        <div className="detail-pane__header-copy">
          <p className="detail-pane__eyebrow">{formatSourceLabel(conversation.sourceApp)}</p>
          <h2>{conversation.title}</h2>
        </div>
      </div>
      <div className="detail-pane__meta">
        <span>{conversation.workspaceName ?? '未识别项目'}</span>
        <span>{formatTimestamp(conversation.updatedAt)}</span>
        <span>{conversation.noteCount} 条笔记</span>
      </div>
      {sourceNotice ? <section className="detail-pane__notice">{sourceNotice}</section> : null}
      {showTimeline ? (
        <section className="detail-pane__section">
          <h3>对话记录</h3>
          <div className="detail-pane__timeline">
            {conversation.messages.length === 0 ? (
              <p className="detail-pane__empty-copy">当前来源暂无可展示的消息时间线。</p>
            ) : (
              conversation.messages.map((message) => {
                const variant = getMessageVariant(message.role, message.messageType);
                const codeLike = isCodeLikeMessage(message.messageType, message.contentText);
                const messageLabel = formatRoleLabel(message.role, message.messageType);
                const canToggleToolMessage = canToggleToolPayload(
                  message.messageType,
                  message.contentText
                );
                const isCollapsed = canToggleToolMessage
                  ? collapsedToolMessages[message.id] ?? false
                  : false;
                const contentId = getMessageContentId(message.id);

                return (
                  <article
                    key={message.id}
                    className="detail-pane__message"
                    data-collapsed={isCollapsed}
                    data-variant={variant}
                  >
                    <div className="detail-pane__message-meta">
                      <span className="detail-pane__message-role">{messageLabel}</span>
                      <span>{formatTimestamp(message.createdAt)}</span>
                      {canToggleToolMessage ? (
                        <button
                          type="button"
                          className="detail-pane__message-toggle"
                          aria-label={isCollapsed ? `展开 ${messageLabel}` : `收起 ${messageLabel}`}
                          aria-controls={contentId}
                          aria-expanded={!isCollapsed}
                          onClick={() =>
                            setCollapsedToolMessages((current) => ({
                              ...current,
                              [message.id]: !isCollapsed
                            }))
                          }
                        >
                          <span
                            aria-hidden="true"
                            className="detail-pane__message-toggle-icon"
                          />
                          {isCollapsed ? `展开` : `收起`}
                        </button>
                      ) : null}
                    </div>
                    {codeLike ? (
                      <div
                        id={contentId}
                        className="detail-pane__message-code-shell"
                        data-collapsed={isCollapsed}
                        data-collapsible={canToggleToolMessage}
                      >
                        <pre className="detail-pane__message-code">{message.contentText}</pre>
                      </div>
                    ) : (
                      <p id={contentId} className="detail-pane__message-text">
                        {message.contentText}
                      </p>
                    )}
                  </article>
                );
              })
            )}
          </div>
        </section>
      ) : (
        <section className="detail-pane__section">
          <h3>对话记录</h3>
          <p className="detail-pane__empty-copy">
            当前来源只同步了会话元数据和提示片段，未恢复完整聊天记录。
          </p>
        </section>
      )}
      <section className="detail-pane__section">
        <h3>摘要预览</h3>
        <p>{conversation.previewText || '暂无摘要预览'}</p>
      </section>
      <details className="detail-pane__metadata">
        <summary>查看原始元数据</summary>
        <pre>{formattedMetadata}</pre>
      </details>
    </section>
  );
}

function getSourceNotice(syncStrength: ConversationDetail['syncStrength']) {
  if (syncStrength === 'partial') {
    return '该来源仅部分可恢复，以下内容可能不包含完整助手回复或工具上下文。';
  }

  if (syncStrength === 'metadata_only') {
    return '当前来源只同步了会话元数据和提示片段，未恢复完整聊天记录。';
  }

  return null;
}

function getMessageVariant(role: string, messageType: string) {
  if (messageType === 'tool_call' || messageType === 'tool_result' || messageType === 'metadata') {
    return 'tool';
  }

  if (role === 'assistant') {
    return 'assistant';
  }

  if (role === 'user') {
    return 'user';
  }

  return 'tool';
}

function formatRoleLabel(role: string, messageType: string) {
  if (messageType === 'tool_call') {
    return 'Tool Call';
  }

  if (messageType === 'tool_result') {
    return 'Tool Result';
  }

  if (messageType === 'metadata') {
    return 'Metadata';
  }

  if (role === 'assistant') {
    return 'Assistant';
  }

  if (role === 'user') {
    return 'User';
  }

  return role;
}

function isCodeLikeMessage(messageType: string, contentText: string) {
  if (messageType === 'tool_call' || messageType === 'tool_result') {
    return true;
  }

  return (
    contentText.includes('```') ||
    (contentText.includes('\n') &&
      /(SELECT|INSERT|UPDATE|DELETE|curl|grpc|import |const |function |\{|\}|=>)/.test(
        contentText
      ))
  );
}

function canToggleToolPayload(messageType: string, contentText: string) {
  if (messageType !== 'tool_call' && messageType !== 'tool_result') {
    return false;
  }

  const lineCount = contentText.split(/\r?\n/).length;
  return lineCount > 10 || contentText.length > 600;
}

function getMessageContentId(messageId: string) {
  return `detail-pane-message-${messageId.replace(/[^a-zA-Z0-9_-]/g, '-')}`;
}

function getResumeCopyLabel(status: ResumeCopyStatus) {
  if (status === 'success') {
    return '已复制';
  }

  if (status === 'error') {
    return '复制失败';
  }

  return '复制 Resume 命令';
}

async function copyText(value: string) {
  if (!navigator.clipboard?.writeText) {
    throw new Error('clipboard unavailable');
  }

  await navigator.clipboard.writeText(value);
}
