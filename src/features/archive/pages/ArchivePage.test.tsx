import { render, screen } from '@testing-library/react';
import { vi } from 'vitest';
import ArchivePage from './ArchivePage';

vi.mock('../../../lib/api', () => ({
  listConversations: vi.fn().mockResolvedValue([])
}));

it('renders an empty-state prompt when no conversations exist', async () => {
  render(<ArchivePage />);

  expect(await screen.findByText('还没有同步任何对话')).toBeInTheDocument();
});
