import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { vi } from 'vitest';
import ArchivePage from './ArchivePage';

const apiMocks = vi.hoisted(() => ({
  listConversations: vi.fn(),
  getArchiveFacets: vi.fn(),
  getConversationDetail: vi.fn(),
  softDeleteConversation: vi.fn()
}));

vi.mock('../../../lib/api', () => ({
  listConversations: apiMocks.listConversations,
  getArchiveFacets: apiMocks.getArchiveFacets,
  getConversationDetail: apiMocks.getConversationDetail,
  softDeleteConversation: apiMocks.softDeleteConversation
}));

function buildConversation(index: number, syncStrength: 'full' | 'partial' = 'full') {
  return {
    id: `${index}`,
    title: `Conversation ${index}`,
    sourceApp: 'codex' as const,
    syncStrength,
    previewText: `Preview ${index}`,
    messageCount: index + 1,
    workspaceName: 'kiko-app',
    updatedAt: index * 1000,
    noteCount: index % 3
  };
}

function buildConversationDetail(conversation: ReturnType<typeof buildConversation>) {
  return {
    id: conversation.id,
    title: conversation.title,
    sourceApp: conversation.sourceApp,
    syncStrength: conversation.syncStrength,
    workspaceName: conversation.workspaceName,
    updatedAt: conversation.updatedAt,
    noteCount: conversation.noteCount,
    previewText: conversation.previewText,
    rawMetadataJson: JSON.stringify({ cwd: '/tmp/kiko-app', conversationId: conversation.id }),
    messages: [
      {
        id: `m-${conversation.id}`,
        role: 'user',
        messageType: 'user',
        contentText: `Message ${conversation.id}`,
        createdAt: conversation.updatedAt
      }
    ]
  };
}

function buildToolPayload(lineCount: number) {
  return Array.from({ length: lineCount }, (_, index) => `tool-line-${index + 1}`).join('\n');
}

it('renders an empty-state prompt when no conversations exist', async () => {
  apiMocks.listConversations.mockResolvedValueOnce([]);
  apiMocks.getArchiveFacets.mockResolvedValueOnce({
    totalCount: 0,
    sourceCounts: []
  });
  render(<ArchivePage />);

  expect(await screen.findByText('还没有同步任何对话')).toBeInTheDocument();
});

it('filters by sync strength and renders the selected conversation detail', async () => {
  const user = userEvent.setup();
  apiMocks.listConversations.mockResolvedValue([
    {
      id: '1',
      title: 'Claude Full',
      sourceApp: 'claude_code',
      syncStrength: 'full',
      previewText: '完整会话预览',
      messageCount: 4,
      workspaceName: 'kiko-front',
      updatedAt: 1000,
      noteCount: 0
    },
    {
      id: '2',
      title: 'CodeX Partial',
      sourceApp: 'codex',
      syncStrength: 'partial',
      previewText: 'CodeX 会话预览',
      messageCount: 2,
      workspaceName: 'kiko-app',
      updatedAt: 2000,
      noteCount: 0
    }
  ]);
  apiMocks.getArchiveFacets.mockResolvedValueOnce({
    totalCount: 2,
    sourceCounts: [
      { sourceApp: 'claude_code', count: 1 },
      { sourceApp: 'codex', count: 1 }
    ]
  });
  apiMocks.getConversationDetail.mockResolvedValue({
    id: '2',
    title: 'CodeX Partial',
    sourceApp: 'codex',
    syncStrength: 'partial',
    workspaceName: 'kiko-app',
    updatedAt: 2000,
    noteCount: 0,
    previewText: 'CodeX 会话预览',
    rawMetadataJson: '{"cwd":"/tmp/kiko-app"}',
    messages: [
      {
        id: 'm1',
        role: 'user',
        messageType: 'user',
        contentText: '排查 gRPC',
        createdAt: 1000
      }
    ]
  });

  render(<ArchivePage />);

  expect(await screen.findByRole('button', { name: /全部对话/i })).toBeInTheDocument();
  await user.click(screen.getByRole('button', { name: 'partial' }));
  await user.click(screen.getByRole('button', { name: 'CodeX Partial' }));

  expect(screen.getByRole('button', { name: 'partial' })).toHaveAttribute('aria-pressed', 'true');
  expect(screen.getByRole('button', { name: 'full' })).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'metadata_only' })).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'CodeX Partial' })).toHaveAttribute(
    'title',
    'CodeX Partial'
  );
  expect(screen.getByRole('heading', { name: 'CodeX Partial' })).toBeInTheDocument();
  expect(screen.getAllByText('CodeX 会话预览')).toHaveLength(2);
  expect(screen.getByText('排查 gRPC')).toBeInTheDocument();
});

it('expands the detail pane while preserving the current conversation and restoring the archive layout on collapse', async () => {
  const user = userEvent.setup();
  apiMocks.listConversations.mockResolvedValue([
    {
      id: '2',
      title: 'CodeX Partial',
      sourceApp: 'codex',
      syncStrength: 'partial',
      previewText: 'CodeX 会话预览',
      messageCount: 2,
      workspaceName: 'kiko-app',
      updatedAt: 2000,
      noteCount: 0
    }
  ]);
  apiMocks.getArchiveFacets.mockResolvedValueOnce({
    totalCount: 1,
    sourceCounts: [{ sourceApp: 'codex', count: 1 }]
  });
  apiMocks.getConversationDetail.mockResolvedValue({
    id: '2',
    title: 'CodeX Partial',
    sourceApp: 'codex',
    syncStrength: 'partial',
    workspaceName: 'kiko-app',
    updatedAt: 2000,
    noteCount: 0,
    previewText: 'CodeX 会话预览',
    rawMetadataJson: '{"cwd":"/tmp/kiko-app"}',
    messages: [
      {
        id: 'm1',
        role: 'user',
        messageType: 'user',
        contentText: '排查 gRPC',
        createdAt: 1000
      }
    ]
  });

  render(<ArchivePage />);

  expect(await screen.findByRole('heading', { name: 'CodeX Partial' })).toBeInTheDocument();
  expect(screen.getByPlaceholderText('搜索标题、项目、关键词')).toBeInTheDocument();

  await user.click(screen.getByRole('button', { name: '展开详情' }));

  const collapseDetailButton = screen.getByRole('button', { name: '收起详情' });
  expect(collapseDetailButton).toBeInTheDocument();
  expect(collapseDetailButton.closest('.detail-pane__sticky-actions')).not.toBeNull();
  expect(collapseDetailButton.closest('.detail-pane__header')).toBeNull();
  expect(screen.queryByPlaceholderText('搜索标题、项目、关键词')).not.toBeInTheDocument();
  expect(screen.queryByText('来源')).not.toBeInTheDocument();
  expect(screen.getByRole('heading', { name: 'CodeX Partial' })).toBeInTheDocument();
  expect(screen.getByText('排查 gRPC')).toBeInTheDocument();

  await user.click(screen.getByRole('button', { name: '收起详情' }));

  const expandDetailButton = screen.getByRole('button', { name: '展开详情' });
  expect(expandDetailButton).toBeInTheDocument();
  expect(expandDetailButton.closest('.detail-pane__sticky-actions')).not.toBeNull();
  expect(expandDetailButton.closest('.detail-pane__header')).toBeNull();
  expect(screen.getByPlaceholderText('搜索标题、项目、关键词')).toBeInTheDocument();
  expect(screen.getByText('来源')).toBeInTheDocument();
  expect(screen.getByRole('heading', { name: 'CodeX Partial' })).toBeInTheDocument();
});

it('keeps long tool messages expanded by default and lets users collapse them without adding controls to short tool messages', async () => {
  const user = userEvent.setup();
  apiMocks.listConversations.mockResolvedValue([
    {
      id: '2',
      title: 'CodeX Partial',
      sourceApp: 'codex',
      syncStrength: 'partial',
      previewText: 'CodeX 会话预览',
      messageCount: 3,
      workspaceName: 'kiko-app',
      updatedAt: 2000,
      noteCount: 0
    }
  ]);
  apiMocks.getArchiveFacets.mockResolvedValueOnce({
    totalCount: 1,
    sourceCounts: [{ sourceApp: 'codex', count: 1 }]
  });
  apiMocks.getConversationDetail.mockResolvedValue({
    id: '2',
    title: 'CodeX Partial',
    sourceApp: 'codex',
    syncStrength: 'partial',
    workspaceName: 'kiko-app',
    updatedAt: 2000,
    noteCount: 0,
    previewText: 'CodeX 会话预览',
    rawMetadataJson: '{"cwd":"/tmp/kiko-app"}',
    messages: [
      {
        id: 'm-user',
        role: 'user',
        messageType: 'user',
        contentText: '排查 gRPC',
        createdAt: 1000
      },
      {
        id: 'm-tool-call',
        role: 'assistant',
        messageType: 'tool_call',
        contentText: buildToolPayload(12),
        createdAt: 1100
      },
      {
        id: 'm-tool-result',
        role: 'assistant',
        messageType: 'tool_result',
        contentText: 'status=ok',
        createdAt: 1200
      }
    ]
  });

  render(<ArchivePage />);

  expect(await screen.findByRole('heading', { name: 'CodeX Partial' })).toBeInTheDocument();

  const longToolMessage = screen.getByText(/tool-line-1/).closest('article');
  expect(longToolMessage).not.toBeNull();

  const longToolToggle = screen.getByRole('button', { name: '收起 Tool Call' });
  expect(longToolToggle).toHaveAttribute('aria-expanded', 'true');
  expect(longToolMessage).toHaveAttribute('data-collapsed', 'false');

  await user.click(longToolToggle);

  expect(screen.getByRole('button', { name: '展开 Tool Call' })).toHaveAttribute(
    'aria-expanded',
    'false'
  );
  expect(longToolMessage).toHaveAttribute('data-collapsed', 'true');

  const shortToolMessage = screen.getByText('status=ok').closest('article');
  expect(shortToolMessage).not.toBeNull();
  expect(
    within(shortToolMessage as HTMLElement).queryByRole('button', { name: /Tool Result/ })
  ).not.toBeInTheDocument();
});

it('only sends search text back through archive queries after the user clicks the search button', async () => {
  const user = userEvent.setup();
  apiMocks.getArchiveFacets.mockResolvedValue({
    totalCount: 1,
    sourceCounts: [{ sourceApp: 'claude_code', count: 1 }]
  });
  apiMocks.getConversationDetail.mockResolvedValue({
    id: '1',
    title: 'Claude Full',
    sourceApp: 'claude_code',
    syncStrength: 'full',
    workspaceName: 'kiko-front',
    updatedAt: 1000,
    noteCount: 0,
    previewText: '完整会话预览',
    rawMetadataJson: '{}',
    messages: []
  });
  apiMocks.listConversations.mockResolvedValue([
    {
      id: '1',
      title: 'Claude Full',
      sourceApp: 'claude_code',
      syncStrength: 'full',
      previewText: '完整会话预览',
      messageCount: 4,
      workspaceName: 'kiko-front',
      updatedAt: 1000,
      noteCount: 0
    }
  ]);

  render(<ArchivePage />);
  await screen.findByRole('button', { name: 'Claude Full' });

  apiMocks.listConversations.mockClear();
  await user.type(screen.getByPlaceholderText('搜索标题、项目、关键词'), 'grpc');

  expect(apiMocks.listConversations).not.toHaveBeenCalled();
  await user.click(screen.getByRole('button', { name: '搜索' }));

  expect(apiMocks.listConversations).toHaveBeenLastCalledWith(
    expect.objectContaining({ text: 'grpc' })
  );
});

it('paginates the middle column at 10 items by default and supports changing page size', async () => {
  const user = userEvent.setup();
  const conversations = Array.from({ length: 12 }, (_, index) => buildConversation(index + 1));

  apiMocks.getArchiveFacets.mockResolvedValueOnce({
    totalCount: conversations.length,
    sourceCounts: [{ sourceApp: 'codex', count: conversations.length }]
  });
  apiMocks.listConversations.mockResolvedValue(conversations);
  apiMocks.getConversationDetail.mockImplementation(async (conversationId: string) =>
    buildConversationDetail(
      conversations.find((conversation) => conversation.id === conversationId) ?? conversations[0]
    )
  );

  render(<ArchivePage />);

  expect(await screen.findByRole('button', { name: 'Conversation 10' })).toBeInTheDocument();
  expect(screen.queryByRole('button', { name: 'Conversation 11' })).not.toBeInTheDocument();
  expect(screen.getByText('第 1 / 2 页')).toBeInTheDocument();
  expect(screen.getByText('当前显示 1-10 / 12 条')).toBeInTheDocument();

  await user.click(screen.getByRole('button', { name: '下一页' }));

  expect(await screen.findByRole('button', { name: 'Conversation 11' })).toBeInTheDocument();
  expect(screen.queryByRole('button', { name: 'Conversation 10' })).not.toBeInTheDocument();
  expect(screen.getByRole('heading', { name: 'Conversation 11' })).toBeInTheDocument();
  expect(screen.getByText('第 2 / 2 页')).toBeInTheDocument();
  expect(screen.getByText('当前显示 11-12 / 12 条')).toBeInTheDocument();

  await user.selectOptions(screen.getByLabelText('每页条数'), '20');

  expect(await screen.findByRole('button', { name: 'Conversation 12' })).toBeInTheDocument();
  expect(screen.getByText('第 1 / 1 页')).toBeInTheDocument();
  expect(screen.getByText('当前显示 1-12 / 12 条')).toBeInTheDocument();
});

it('resets pagination to the first page when filters change', async () => {
  const user = userEvent.setup();
  const conversations = [
    ...Array.from({ length: 6 }, (_, index) => buildConversation(index + 1, 'full')),
    ...Array.from({ length: 6 }, (_, index) => buildConversation(index + 7, 'partial'))
  ];

  apiMocks.getArchiveFacets.mockResolvedValueOnce({
    totalCount: conversations.length,
    sourceCounts: [{ sourceApp: 'codex', count: conversations.length }]
  });
  apiMocks.listConversations.mockImplementation(async (query?: { syncStrength?: string }) => {
    if (!query?.syncStrength) {
      return conversations;
    }

    return conversations.filter((conversation) => conversation.syncStrength === query.syncStrength);
  });
  apiMocks.getConversationDetail.mockImplementation(async (conversationId: string) =>
    buildConversationDetail(
      conversations.find((conversation) => conversation.id === conversationId) ?? conversations[0]
    )
  );

  render(<ArchivePage />);

  expect(await screen.findByRole('button', { name: 'Conversation 10' })).toBeInTheDocument();
  await user.click(screen.getByRole('button', { name: '下一页' }));

  expect(await screen.findByRole('heading', { name: 'Conversation 11' })).toBeInTheDocument();
  expect(screen.getByText('第 2 / 2 页')).toBeInTheDocument();

  await user.click(screen.getByRole('button', { name: 'partial' }));

  expect(await screen.findByText('第 1 / 1 页')).toBeInTheDocument();
  expect(screen.getByText('当前显示 1-6 / 6 条')).toBeInTheDocument();
  expect(await screen.findByRole('heading', { name: 'Conversation 7' })).toBeInTheDocument();
  expect(screen.queryByRole('button', { name: 'Conversation 1' })).not.toBeInTheDocument();
});

it('opens a delete confirmation dialog and does not delete when cancelled', async () => {
  const user = userEvent.setup();
  const conversations = [buildConversation(1), buildConversation(2)];

  apiMocks.getArchiveFacets.mockResolvedValueOnce({
    totalCount: conversations.length,
    sourceCounts: [{ sourceApp: 'codex', count: conversations.length }]
  });
  apiMocks.listConversations.mockResolvedValue(conversations);
  apiMocks.getConversationDetail.mockImplementation(async (conversationId: string) =>
    buildConversationDetail(
      conversations.find((conversation) => conversation.id === conversationId) ?? conversations[0]
    )
  );

  render(<ArchivePage />);

  expect(await screen.findByRole('button', { name: 'Conversation 1' })).toBeInTheDocument();

  await user.click(screen.getByRole('button', { name: '删除 Conversation 1' }));

  expect(screen.getByRole('dialog', { name: '确认删除历史记录' })).toBeInTheDocument();
  expect(
    screen.getByText(/仅从 Context Vault 已同步数据库中隐藏，不删除本地 Claude\/Codex 原始记录/)
  ).toBeInTheDocument();

  await user.click(screen.getByRole('button', { name: '取消' }));

  await waitFor(() => {
    expect(
      screen.queryByRole('dialog', { name: '确认删除历史记录' })
    ).not.toBeInTheDocument();
  });
  expect(apiMocks.softDeleteConversation).not.toHaveBeenCalled();
});

it('deletes the selected conversation and selects the next visible item', async () => {
  const user = userEvent.setup();
  const initialConversations = [buildConversation(1), buildConversation(2)];
  const remainingConversations = [buildConversation(2)];

  apiMocks.getArchiveFacets
    .mockResolvedValueOnce({
      totalCount: initialConversations.length,
      sourceCounts: [{ sourceApp: 'codex', count: initialConversations.length }]
    })
    .mockResolvedValueOnce({
      totalCount: remainingConversations.length,
      sourceCounts: [{ sourceApp: 'codex', count: remainingConversations.length }]
    });
  apiMocks.listConversations
    .mockResolvedValueOnce(initialConversations)
    .mockResolvedValueOnce(remainingConversations);
  apiMocks.getConversationDetail.mockImplementation(async (conversationId: string) =>
    buildConversationDetail(
      [...initialConversations, ...remainingConversations].find(
        (conversation) => conversation.id === conversationId
      ) ?? remainingConversations[0]
    )
  );
  apiMocks.softDeleteConversation.mockResolvedValue(undefined);

  render(<ArchivePage />);

  expect(await screen.findByRole('heading', { name: 'Conversation 1' })).toBeInTheDocument();

  await user.click(screen.getByRole('button', { name: '删除 Conversation 1' }));
  await user.click(screen.getByRole('button', { name: '删除' }));

  await waitFor(() => {
    expect(apiMocks.softDeleteConversation).toHaveBeenCalledWith('1');
  });
  await waitFor(() => {
    expect(screen.queryByRole('button', { name: 'Conversation 1' })).not.toBeInTheDocument();
  });
  expect(await screen.findByRole('heading', { name: 'Conversation 2' })).toBeInTheDocument();
  expect(screen.getByText('当前显示 1-1 / 1 条')).toBeInTheDocument();
});
