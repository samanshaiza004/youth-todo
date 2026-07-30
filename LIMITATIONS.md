# Limitations

Youth Todo is a collection architecture probe, not a production task editor.

- Titles are generated as `Task N`; there is no text entry.
- At most 64 live tasks are supported and five are visible per page.
- There is no scrolling, drag-and-drop, priority, due date, nesting, search,
  synchronization, import, or export.
- Filter and page are intentionally process-local and reset to All/page zero on
  restart.
- Persistence uses explicit canonical keys. State enumeration, typed records,
  scoped keys, structured documents, and collection transactions are absent.
- Presentation uses explicit structural patches. SDK tree diffing and reactive
  dependencies are absent.
- The renderer remains provisional, mouse/keyboard policy is host-owned, and
  accessibility projection is not yet available.
