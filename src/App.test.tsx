import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { vi } from 'vitest';
import App from './App';

const apiMocks = vi.hoisted(() => ({
  runSync: vi.fn()
}));

const viewMounts = vi.hoisted(() => ({
  archive: 0,
  notes: 0,
  sync: 0,
  settings: 0
}));

vi.mock('./lib/api', () => ({
  runSync: apiMocks.runSync
}));

vi.mock('./features/archive/pages/ArchivePage', async () => {
  const React = await import('react');

  return {
    default: ({ refreshKey = 0 }: { refreshKey?: number }) => {
      React.useEffect(() => {
        viewMounts.archive += 1;
      }, []);

      return <section>Archive refresh {refreshKey}</section>;
    }
  };
});

vi.mock('./features/notes/pages/NotesPage', async () => {
  const React = await import('react');

  return {
    default: () => {
      React.useEffect(() => {
        viewMounts.notes += 1;
      }, []);

      return <section>Notes</section>;
    }
  };
});

vi.mock('./features/sync/pages/SyncLogPage', async () => {
  const React = await import('react');

  return {
    default: () => {
      React.useEffect(() => {
        viewMounts.sync += 1;
      }, []);

      return <section>Sync</section>;
    }
  };
});

vi.mock('./features/settings/pages/SettingsPage', async () => {
  const React = await import('react');

  return {
    default: () => {
      React.useEffect(() => {
        viewMounts.settings += 1;
      }, []);

      return <section>Settings</section>;
    }
  };
});

function createDeferred() {
  let resolve!: () => void;
  const promise = new Promise<void>((res) => {
    resolve = res;
  });

  return { promise, resolve };
}

it('shows a loading sync button while a manual sync is in flight', async () => {
  const user = userEvent.setup();
  const pendingSync = createDeferred();

  apiMocks.runSync.mockResolvedValueOnce(undefined);
  apiMocks.runSync.mockReturnValueOnce(pendingSync.promise);

  render(<App />);

  await waitFor(() =>
    expect(screen.getByRole('button', { name: '同步' })).not.toBeDisabled()
  );

  await user.click(screen.getByRole('button', { name: '同步' }));

  const syncingButton = screen.getByRole('button', { name: '同步中' });
  expect(syncingButton).toBeDisabled();
  expect(syncingButton).toHaveAttribute('aria-busy', 'true');

  pendingSync.resolve();

  await waitFor(() =>
    expect(screen.getByRole('button', { name: '同步' })).not.toBeDisabled()
  );
});

it('keeps visited pages mounted when switching between notes and archive', async () => {
  const user = userEvent.setup();

  viewMounts.archive = 0;
  viewMounts.notes = 0;
  apiMocks.runSync.mockResolvedValue(undefined);

  render(<App />);

  await screen.findByText('Archive refresh 1');
  expect(viewMounts.archive).toBe(1);

  await user.click(screen.getByRole('button', { name: '知识笔记' }));
  await screen.findByText('Notes');
  expect(viewMounts.notes).toBe(1);

  await user.click(screen.getByRole('button', { name: '对话记录' }));
  await screen.findByText('Archive refresh 1');

  expect(viewMounts.archive).toBe(1);
  expect(viewMounts.notes).toBe(1);
});

it('uses contained scrolling for archive view and page scrolling for other views', async () => {
  const user = userEvent.setup();

  apiMocks.runSync.mockResolvedValue(undefined);

  render(<App />);

  await screen.findByText('Archive refresh 1');
  expect(document.querySelector('.content')).toHaveAttribute('data-scroll-mode', 'contained');

  await user.click(screen.getByRole('button', { name: '知识笔记' }));

  await screen.findByText('Notes');
  expect(document.querySelector('.content')).toHaveAttribute('data-scroll-mode', 'page');

  await user.click(screen.getByRole('button', { name: '对话记录' }));

  await screen.findByText('Archive refresh 1');
  expect(document.querySelector('.content')).toHaveAttribute('data-scroll-mode', 'contained');
});
