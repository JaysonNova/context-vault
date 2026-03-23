import { useEffect, useState } from 'react';
import { listNotes } from '../../../lib/api';
import type { NoteListItem } from '../../../lib/types';

type NotesPageProps = {
  refreshKey?: number;
};

export default function NotesPage({ refreshKey = 0 }: NotesPageProps) {
  const [notes, setNotes] = useState<NoteListItem[]>([]);

  useEffect(() => {
    let isMounted = true;

    void listNotes().then((items) => {
      if (isMounted) {
        setNotes(items);
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
          <p className="panel-page__eyebrow">Knowledge Base</p>
          <h2>已沉淀笔记</h2>
        </div>
        <button type="button">导出 Markdown</button>
      </div>
      <div className="panel-page__grid">
        {notes.map((note) => (
          <article key={note.id} className="panel-card">
            <h3>{note.title}</h3>
            <p>{note.summary}</p>
          </article>
        ))}
      </div>
    </section>
  );
}
