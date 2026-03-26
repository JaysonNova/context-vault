/// <reference types="node" />

import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { render, screen } from '@testing-library/react';
import ConversationFilters from './ConversationFilters';

const styles = readFileSync(resolve(process.cwd(), 'src/styles.css'), 'utf8');

describe('ConversationFilters', () => {
  it('renders the current result count summary', () => {
    render(
      <ConversationFilters
        searchText=""
        selectedSyncStrength={undefined}
        resultCount={237}
        onSearchTextChange={() => undefined}
        onSearchSubmit={() => undefined}
        onSyncStrengthChange={() => undefined}
      />
    );

    expect(screen.getByText('237 条结果')).toBeInTheDocument();
  });

  it('uses a fixed desktop summary column so larger counts do not squeeze the search area', () => {
    expect(styles).toMatch(
      /\.conversation-filters\s*\{[\s\S]*grid-template-columns:\s*minmax\(0,\s*1fr\)\s+var\(--conversation-filters-summary-width\);/
    );
    expect(styles).toMatch(
      /\.conversation-filters\s*\{[\s\S]*--conversation-filters-summary-width:\s*7\.5rem;/
    );
  });
});
