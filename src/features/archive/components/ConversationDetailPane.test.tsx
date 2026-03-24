import { render, screen } from '@testing-library/react';
import ConversationDetailPane from './ConversationDetailPane';
import type { ConversationDetail } from '../../../lib/types';

function buildConversationDetail(
  overrides: Partial<ConversationDetail> = {}
): ConversationDetail {
  return {
    id: 'conv_1',
    title: 'gRPC 连接池耗尽排查',
    sourceApp: 'claude_code',
    syncStrength: 'full',
    workspaceName: 'kiko-app',
    updatedAt: 1711111111000,
    noteCount: 1,
    previewText: '根因是并发流上限过低',
    rawMetadataJson: '{"cwd":"/tmp/kiko-app"}',
    messages: [
      {
        id: 'm1',
        role: 'user',
        messageType: 'user',
        contentText: '排查 gRPC RESOURCE_EXHAUSTED 的根因',
        createdAt: 1711111111000
      },
      {
        id: 'm2',
        role: 'assistant',
        messageType: 'assistant',
        contentText: '我先检查当前 gRPC 服务端配置。',
        createdAt: 1711111112000
      },
      {
        id: 'm3',
        role: 'tool',
        messageType: 'tool_result',
        contentText: 'server := grpc.NewServer(grpc.MaxConcurrentStreams(100))',
        createdAt: 1711111113000
      }
    ],
    ...overrides
  };
}

it('renders a chat-style timeline for full conversations', () => {
  render(<ConversationDetailPane conversation={buildConversationDetail()} />);

  expect(screen.getByText('对话记录')).toBeInTheDocument();
  expect(screen.getByText('排查 gRPC RESOURCE_EXHAUSTED 的根因')).toBeInTheDocument();
  expect(screen.getByText('我先检查当前 gRPC 服务端配置。')).toBeInTheDocument();
  expect(
    screen.getByText('server := grpc.NewServer(grpc.MaxConcurrentStreams(100))')
  ).toBeInTheDocument();
});

it('shows a partial-source notice and keeps rendered messages', () => {
  render(
    <ConversationDetailPane
      conversation={buildConversationDetail({
        sourceApp: 'codex',
        syncStrength: 'partial'
      })}
    />
  );

  expect(screen.getByText(/该来源仅部分可恢复/)).toBeInTheDocument();
  expect(screen.getByText('排查 gRPC RESOURCE_EXHAUSTED 的根因')).toBeInTheDocument();
});

it('shows metadata-only guidance when no reliable timeline exists', () => {
  render(
    <ConversationDetailPane
      conversation={buildConversationDetail({
        sourceApp: 'cursor',
        syncStrength: 'metadata_only',
        messages: []
      })}
    />
  );

  expect(screen.getAllByText(/当前来源只同步了会话元数据和提示片段/)).toHaveLength(2);
  expect(screen.queryByText('消息时间线')).not.toBeInTheDocument();
});
