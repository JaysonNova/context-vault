import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { vi } from 'vitest';
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
    resumeCommand: undefined,
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

it('renders a codex resume copy button in the sticky action row and copies the command', async () => {
  const user = userEvent.setup();
  const writeText = vi.fn().mockResolvedValue(undefined);
  Object.defineProperty(window.navigator, 'clipboard', {
    configurable: true,
    value: { writeText }
  });

  render(
    <ConversationDetailPane
      conversation={buildConversationDetail({
        sourceApp: 'codex',
        syncStrength: 'partial',
        resumeCommand: "codex resume -C '/tmp/kiko-app' session-1"
      })}
      onToggleExpanded={() => {}}
    />
  );

  const copyButton = screen.getByRole('button', { name: '复制 Resume 命令' });
  expect(copyButton.closest('.detail-pane__sticky-actions')).not.toBeNull();

  await user.click(copyButton);

  expect(writeText).toHaveBeenCalledWith("codex resume -C '/tmp/kiko-app' session-1");
  await waitFor(() => {
    expect(screen.getByRole('button', { name: '已复制' })).toBeInTheDocument();
  });
});

it('renders a claude resume copy button and copies the path-aware command', async () => {
  const user = userEvent.setup();
  const writeText = vi.fn().mockResolvedValue(undefined);
  Object.defineProperty(window.navigator, 'clipboard', {
    configurable: true,
    value: { writeText }
  });

  render(
    <ConversationDetailPane
      conversation={buildConversationDetail({
        sourceApp: 'claude_code',
        syncStrength: 'full',
        resumeCommand: "cd '/tmp/kiko-app' && claude --resume session-1"
      })}
    />
  );

  await user.click(screen.getByRole('button', { name: '复制 Resume 命令' }));

  expect(writeText).toHaveBeenCalledWith("cd '/tmp/kiko-app' && claude --resume session-1");
  await waitFor(() => {
    expect(screen.getByRole('button', { name: '已复制' })).toBeInTheDocument();
  });
});

it('resets copy feedback when the selected conversation changes and hides the action for non-codex sources', async () => {
  const user = userEvent.setup();
  const writeText = vi.fn().mockResolvedValue(undefined);
  Object.defineProperty(window.navigator, 'clipboard', {
    configurable: true,
    value: { writeText }
  });

  const { rerender } = render(
    <ConversationDetailPane
      conversation={buildConversationDetail({
        id: 'codex-conv',
        sourceApp: 'codex',
        syncStrength: 'partial',
        resumeCommand: 'codex resume session-1'
      })}
    />
  );

  await user.click(screen.getByRole('button', { name: '复制 Resume 命令' }));
  await waitFor(() => {
    expect(screen.getByRole('button', { name: '已复制' })).toBeInTheDocument();
  });

  rerender(
    <ConversationDetailPane
      conversation={buildConversationDetail({
        id: 'claude-conv',
        sourceApp: 'claude_code',
        syncStrength: 'full',
        resumeCommand: undefined
      })}
    />
  );

  expect(screen.queryByRole('button', { name: '复制 Resume 命令' })).not.toBeInTheDocument();
  expect(screen.queryByRole('button', { name: '已复制' })).not.toBeInTheDocument();
});

it('shows failure feedback when copying the resume command fails', async () => {
  const user = userEvent.setup();
  const writeText = vi.fn().mockRejectedValue(new Error('clipboard unavailable'));
  Object.defineProperty(window.navigator, 'clipboard', {
    configurable: true,
    value: { writeText }
  });

  render(
    <ConversationDetailPane
      conversation={buildConversationDetail({
        sourceApp: 'codex',
        syncStrength: 'partial',
        resumeCommand: 'codex resume session-1'
      })}
    />
  );

  await user.click(screen.getByRole('button', { name: '复制 Resume 命令' }));

  await waitFor(() => {
    expect(screen.getByRole('button', { name: '复制失败' })).toBeInTheDocument();
  });
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
