import { render, screen } from '@testing-library/react';
import { vi } from 'vitest';
import SyncLogPage from './SyncLogPage';

vi.mock('../../../lib/api', () => ({
  listSyncRuns: vi.fn().mockResolvedValue([
    {
      sourceApp: 'claude_code',
      status: 'success',
      importedConversationCount: 3
    },
    {
      sourceApp: 'cursor',
      status: 'failed',
      importedConversationCount: 0,
      errorSummary: 'db locked'
    }
  ])
}));

it('shows per-source sync results', async () => {
  render(<SyncLogPage />);

  expect(await screen.findByText('claude_code')).toBeInTheDocument();
  expect(screen.getByText('db locked')).toBeInTheDocument();
});
