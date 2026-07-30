# Youth Todo

Todo is Youth Utility Suite's bounded dynamic-collection probe. It supports up
to 64 generated tasks, stable non-reused identities, five-row paging, filters,
ordering, durable migration, and explicit structural updates on
`youth:app@0.0.5`.

```bash
/path/to/youth check
/path/to/youth test --verify-view-convergence
/path/to/youth build --release
/path/to/youth dev
```

The project pins the immutable collections SDK revision in both Cargo and
`Youth.lock`; it has no dependency on a local Youth checkout. See
`FINDINGS.md` for architecture evidence and `LIMITATIONS.md` for deliberate
scope boundaries.

## License

Licensed under Apache-2.0 or MIT at your option.
