import { useEffect, useState } from 'react';
import { listSyncRuns } from '../../../lib/api';
import type { SyncRunListItem } from '../../../lib/types';

type SyncLogPageProps = {
  refreshKey?: number;
};

export default function SyncLogPage({ refreshKey = 0 }: SyncLogPageProps) {
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
  }, [refreshKey]);

  return (
    <section>
      {runs.map((run, index) => (
        <article key={`${run.sourceApp}-${run.status}-${index}`}>
          <h2>{run.sourceApp}</h2>
          <p>{run.status}</p>
          <p>{run.importedConversationCount}</p>
          {run.errorSummary ? <p>{run.errorSummary}</p> : null}
        </article>
      ))}
    </section>
  );
}
