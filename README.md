# Youth Todo

Todo is Youth Utility Suite's bounded dynamic-collection probe. It supports up
to 64 generated tasks, stable non-reused identities, five-row paging, filters,
ordering, durable migration, and explicit structural updates on
`youth:app@0.0.5`.

Install the matching Youth CLI from the immutable release tag, then run the
project commands from this directory:

```bash
cargo install youth-cli --git https://github.com/samanshaiza004/youth \
  --tag utility-todo-gate-d-release
youth doctor
youth check
youth test --verify-view-convergence
youth build --release
youth dev
```

The project pins the immutable collections SDK revision in both Cargo and
`Youth.lock`; it has no dependency on a local Youth checkout. See
`FINDINGS.md` for architecture evidence and `LIMITATIONS.md` for deliberate
scope boundaries.

The release uses one canonical validated component on every supported host.
That artifact is 281,660 bytes with SHA-256
`d92f6f1aa9c8fa945cd4c087284a67c001b0c853926adb53c269d56d393fac52`.
Host-local source builds are portability evidence and are not expected to have
the same bytes.

## License

Licensed under Apache-2.0 or MIT at your option.
