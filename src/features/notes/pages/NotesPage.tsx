import { useEffect, useState } from 'react';
import { listNotes } from '../../../lib/api';
import type { NoteListItem } from '../../../lib/types';

export default function NotesPage() {
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
  }, []);

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
