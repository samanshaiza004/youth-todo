# Todo release metrics

Release measurements below were collected on 2026-07-30 with Rust 1.97.1,
Youth `42c606b5b6549d3e1b100868d1750d3155311578`, SDK
`844102bbeadfbbf135e4b9da36423bdc435fbb16`, and the Todo Gate D source. Local
macOS/aarch64 values are evidence, not cross-platform performance claims.

| Metric | Result | Evidence / limitation |
| --- | ---: | --- |
| Debug component | 9,209,346 bytes | `youth check` development artifact |
| Release component | 281,657 bytes | `youth build --release` |
| Local release component SHA-256 | `e30eac4c65520ad896b27aa4b379713bce05e9764d8b65f349a1ec499c69f9ca` | macOS source build; not the canonical release identity |
| Canonical release component | 281,660 bytes | Ubuntu canonical builder retained these exact bytes |
| Canonical component SHA-256 | `d92f6f1aa9c8fa945cd4c087284a67c001b0c853926adb53c269d56d393fac52` | Expected and observed before every certified host mount |
| Youth host executable | 24,834,560 bytes | local `target/release/youth`; install-prefix overhead was not separately measured |
| Raw-WIT concepts exposed to app source | 0 | source boundary scan plus review |
| Crash rollback | Passed | legacy/current Todo-shaped injected commit failure retains state, tree, focus, and emits no observer event |
| Same source builds on supported hosts | Passed | locked source builds and validates on Ubuntu, Windows, and macOS; local hashes are logged but need not match |
| One canonical component on all hosts | Passed | the exact canonical digest above mounts on Ubuntu, Windows, and macOS |
| Representative state writes | 5 for the first Add | schema, next ID, order, title, and status; full-model persistence repetition remains TODO-F004 evidence |
| Presentation patch counts | Unavailable | primitive patch counts are not exported by the release harness; reporting the SDK operation count would understate wire patches |
| Accessibility completeness | Unavailable | Youth has no accessibility projection yet; reporting zero would imply a completed inventory |
| Fresh install to first window | Unavailable | no controlled clean-machine sample was collected |
| `youth new` to app run | Unavailable | Todo was manually assembled; `youth new` still generates Tally |
| Cold start / first present | Unavailable | runtime stages are not yet exported by the release harness |
| Memory per loaded app | Unavailable | no controlled RSS harness exists across all three hosts |
| Idle CPU / guest calls / redraws | Unavailable | idle call/redraw invariants are tested, but CPU sampling is not standardized |
| Turn latency / presentation latency | Unavailable | spans exist, but no release distribution aggregator exists |
| Boundary bytes per event | Unavailable | component-boundary byte accounting is not instrumented |
| State commit latency | Unavailable | commit spans exist, but no release distribution aggregator exists |
| Timed crash recovery | Unavailable | correctness is proven; recovery duration was not sampled |

The canonical artifact manifest produced in Youth CI records the exact Todo,
Youth, SDK, protocol, WIT, toolchain, byte-size, and component-hash identity.
It distinguishes canonical artifact portability, host-local source builds, and
runtime compatibility. Reproducible independent builds are not claimed.
