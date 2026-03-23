import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { vi } from 'vitest';
import ArchivePage from './ArchivePage';

const apiMocks = vi.hoisted(() => ({
  listConversations: vi.fn(),
  getArchiveFacets: vi.fn(),
  getConversationDetail: vi.fn()
}));

vi.mock('../../../lib/api', () => ({
  listConversations: apiMocks.listConversations,
  getArchiveFacets: apiMocks.getArchiveFacets,
  getConversationDetail: apiMocks.getConversationDetail
}));

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
  expect(screen.getByRole('heading', { name: 'CodeX Partial' })).toBeInTheDocument();
  expect(screen.getAllByText('CodeX 会话预览')).toHaveLength(2);
  expect(screen.getByText('排查 gRPC')).toBeInTheDocument();
});

it('sends search text back through archive queries', async () => {
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

  await user.type(screen.getByPlaceholderText('搜索标题、项目、关键词'), 'grpc');

  expect(apiMocks.listConversations).toHaveBeenLastCalledWith(
    expect.objectContaining({ text: 'grpc' })
  );
});
