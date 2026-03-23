import { render, screen } from '@testing-library/react';
import { vi } from 'vitest';
import NotesPage from './NotesPage';

const apiMocks = vi.hoisted(() => ({
  listNotes: vi.fn()
}));

vi.mock('../../../lib/api', () => ({
  listNotes: apiMocks.listNotes
}));

it('renders generated notes and export action', async () => {
  apiMocks.listNotes.mockResolvedValueOnce([
    { id: 'n1', title: 'gRPC 排障笔记', summary: '连接池上限问题' }
  ]);

  render(<NotesPage />);

  expect(await screen.findByText('gRPC 排障笔记')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: '导出 Markdown' })).toBeInTheDocument();
});
