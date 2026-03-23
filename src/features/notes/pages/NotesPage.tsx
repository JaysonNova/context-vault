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
    <section>
      <button type="button">导出 Markdown</button>
      {notes.map((note) => (
        <article key={note.id}>
          <h2>{note.title}</h2>
          <p>{note.summary}</p>
        </article>
      ))}
    </section>
  );
}
