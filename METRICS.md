# Todo release metrics

Release candidate measurements below were collected on 2026-07-30 from
macOS/aarch64 with Rust 1.97.1, Youth
`9b562af2946a1f3489bfc61f42bb0f7ae498d62d`, SDK
`844102bbeadfbbf135e4b9da36423bdc435fbb16`, and Todo `fee04a5`. A single
local value is evidence, not a cross-platform performance claim.

| Metric | Result | Evidence / limitation |
| --- | ---: | --- |
| Debug component | 9,209,346 bytes | `youth check` development artifact |
| Release component | 281,657 bytes | `youth build --release` |
| Local release component SHA-256 | `e30eac4c65520ad896b27aa4b379713bce05e9764d8b65f349a1ec499c69f9ca` | macOS source build; not the canonical release identity |
| Youth host executable | 24,834,560 bytes | local `target/release/youth`; install-prefix overhead was not separately measured |
| Raw-WIT concepts exposed to app source | 0 | source boundary scan plus review |
| Crash rollback | Passed | legacy/current Todo-shaped injected commit failure retains state, tree, focus, and emits no observer event |
| Same source builds on supported hosts | Pending CI | host-local hashes are logged but need not match |
| One canonical component on all hosts | Pending CI | canonical Ubuntu artifact is certified by Youth's host matrix |
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
