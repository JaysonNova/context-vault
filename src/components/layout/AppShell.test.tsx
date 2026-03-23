import { render, screen } from '@testing-library/react';
import AppShell from './AppShell';

it('renders the primary navigation and sync action', () => {
  render(<AppShell />);

  expect(screen.getByText('对话记录')).toBeInTheDocument();
  expect(screen.getByText('知识笔记')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: '同步' })).toBeInTheDocument();
});
