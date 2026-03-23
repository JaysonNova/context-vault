import { useEffect, useState } from 'react';
import { listSyncRuns } from '../../../lib/api';
import type { SyncRunListItem } from '../../../lib/types';

export default function SyncLogPage() {
  const [runs, setRuns] = useState<SyncRunListItem[]>([]);

  useEffect(() => {
    let isMounted = true;

    void listSyncRuns().then((items) => {
      if (isMounted) {
        setRuns(items);
      }
    });

    return () => {
      isMounted = false;
    };
  }, []);

  return (
    <section>
      {runs.map((run) => (
        <article key={`${run.sourceApp}-${run.status}`}>
          <h2>{run.sourceApp}</h2>
          <p>{run.status}</p>
          <p>{run.importedConversationCount}</p>
          {run.errorSummary ? <p>{run.errorSummary}</p> : null}
        </article>
      ))}
    </section>
  );
}
