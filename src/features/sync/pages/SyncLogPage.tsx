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
    <section className="panel-page">
      <div className="panel-page__header">
        <div>
          <p className="panel-page__eyebrow">Sync Runs</p>
          <h2>最近同步</h2>
        </div>
      </div>
      {runs.map((run, index) => (
        <article key={`${run.sourceApp}-${run.status}-${index}`} className="panel-card panel-card--sync">
          <h2>{run.sourceApp}</h2>
          <p>{run.status}</p>
          <p>{run.importedConversationCount}</p>
          {run.errorSummary ? <p>{run.errorSummary}</p> : null}
        </article>
      ))}
    </section>
  );
}
