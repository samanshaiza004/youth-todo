# Youth Todo

The third Youth Utility Suite application. Todo is deliberately a dynamic
collection experiment rather than a text-editing application: task titles are
generated (`Task 1`, `Task 2`, …), at most 64 tasks are durable, and five rows
are presented per filtered page.

Gate A freezes the domain model and explicit durable codec, then demonstrates
the published SDK blockers for runtime-derived identities and structural tree
updates. See `FINDINGS.md`.

The final application will use protocol `0.0.5` without raw WIT bindings,
numeric IDs, revisions, acknowledgements, patches, or export plumbing.

## License

Licensed under Apache-2.0 or MIT at your option.
