/// <reference types="node" />

import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { vi } from 'vitest';
import ConversationList from './ConversationList';

const styles = readFileSync(resolve(process.cwd(), 'src/styles.css'), 'utf8');

describe('ConversationList', () => {
  it('renders the conversation preview text', () => {
    render(
      <ConversationList
        conversations={[
          {
            id: '1',
            title: 'Conversation 1',
            sourceApp: 'cursor',
            syncStrength: 'partial',
            workspaceName: 'context-vault',
            updatedAt: 1_774_000_000_000,
            messageCount: 12,
            previewText: '这是一段很长的摘要预览内容',
            noteCount: 1
          }
        ]}
        selectedConversationId="1"
        onSelect={() => undefined}
      />
    );

    expect(screen.getByText('这是一段很长的摘要预览内容')).toBeInTheDocument();
  });

  it('clamps preview text to 5 lines by default and keeps the line count configurable via css variable', () => {
    expect(styles).toContain('--conversation-list-preview-lines: 5;');
    expect(styles).toContain('-webkit-line-clamp: var(--conversation-list-preview-lines);');
    expect(styles).toContain('max-height: calc(1.55em * var(--conversation-list-preview-lines));');
    expect(styles).toContain('display: -webkit-box;');
    expect(styles).toContain('overflow: hidden;');
  });

  it('shows delete actions only for codex and claude rows and does not select a row when deleting', async () => {
    const user = userEvent.setup();
    const onSelect = vi.fn();
    const onDeleteRequest = vi.fn();

    render(
      <ConversationList
        conversations={[
          {
            id: 'codex-1',
            title: 'Codex Conversation',
            sourceApp: 'codex',
            syncStrength: 'partial',
            workspaceName: 'context-vault',
            updatedAt: 1_774_000_000_000,
            messageCount: 12,
            previewText: 'codex preview',
            noteCount: 1
          },
          {
            id: 'claude-1',
            title: 'Claude Conversation',
            sourceApp: 'claude_code',
            syncStrength: 'full',
            workspaceName: 'context-vault',
            updatedAt: 1_774_000_000_000,
            messageCount: 8,
            previewText: 'claude preview',
            noteCount: 0
          },
          {
            id: 'cursor-1',
            title: 'Cursor Conversation',
            sourceApp: 'cursor',
            syncStrength: 'metadata_only',
            workspaceName: 'context-vault',
            updatedAt: 1_774_000_000_000,
            messageCount: 1,
            previewText: 'cursor preview',
            noteCount: 0
          }
        ]}
        selectedConversationId="codex-1"
        onSelect={onSelect}
        onDeleteRequest={onDeleteRequest}
      />
    );

    expect(screen.getByRole('button', { name: '删除 Codex Conversation' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '删除 Claude Conversation' })).toBeInTheDocument();
    expect(
      screen.queryByRole('button', { name: '删除 Cursor Conversation' })
    ).not.toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: '删除 Codex Conversation' }));

    expect(onDeleteRequest).toHaveBeenCalledWith(
      expect.objectContaining({ id: 'codex-1', sourceApp: 'codex' })
    );
    expect(onSelect).not.toHaveBeenCalled();
  });

  it('reveals delete actions on hover and focus-within', () => {
    expect(styles).toContain(".conversation-list__item:hover .conversation-list__delete");
    expect(styles).toContain(".conversation-list__item:focus-within .conversation-list__delete");
  });
});
