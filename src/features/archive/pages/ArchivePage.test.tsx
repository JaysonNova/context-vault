import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { vi } from 'vitest';
import ArchivePage from './ArchivePage';

const apiMocks = vi.hoisted(() => ({
  listConversations: vi.fn()
}));

vi.mock('../../../lib/api', () => ({
  listConversations: apiMocks.listConversations
}));

it('renders an empty-state prompt when no conversations exist', async () => {
  apiMocks.listConversations.mockResolvedValueOnce([]);
  render(<ArchivePage />);

  expect(await screen.findByText('还没有同步任何对话')).toBeInTheDocument();
});

it('filters by sync strength and renders the selected conversation detail', async () => {
  const user = userEvent.setup();
  apiMocks.listConversations.mockResolvedValueOnce([
    {
      id: '1',
      title: 'Claude Full',
      sourceApp: 'claude_code',
      syncStrength: 'full',
      updatedAt: 1000,
      noteCount: 0
    },
    {
      id: '2',
      title: 'CodeX Partial',
      sourceApp: 'codex',
      syncStrength: 'partial',
      updatedAt: 2000,
      noteCount: 0
    }
  ]);

  render(<ArchivePage />);

  await screen.findByRole('button', { name: 'Claude Full' });
  await user.click(screen.getByRole('button', { name: 'partial' }));
  await user.click(screen.getByRole('button', { name: 'CodeX Partial' }));

  expect(screen.getByRole('button', { name: 'partial' })).toHaveAttribute('aria-pressed', 'true');
  expect(screen.getByRole('heading', { name: 'CodeX Partial' })).toBeInTheDocument();
});
