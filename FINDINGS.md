# Todo Findings

This is the canonical evidence record for Youth Todo. Findings are evidence,
not automatic authorization for platform features. Local experiments were run
from `/Users/keina/dev/youth-todo` on macOS 15/aarch64 on 2026-07-30.

## Findings index

| ID | Category | Observation | Status | Decision |
| --- | --- | --- | --- | --- |
| TODO-F001 | Platform discovery | Static-only SDK keys could not represent durable runtime item identities | Addressed at Gate B | `ItemKey` derives typed node/command identities; protocol remains `0.0.5` |
| TODO-F002 | Platform discovery | SDK property updates could not insert, remove, or move rows although WIT already had primitives | Addressed at Gate B | Named containers and strict SDK subtree operations expand into existing patches |
| TODO-F003 | Application convention | Filter/page are process session state, but the lifecycle exposes no app instance object | Deferred | A small app-owned `thread_local RefCell<TodoSession>` is sufficient evidence; do not persist navigation |
| TODO-F004 | Platform discovery | Explicit item persistence repeats key construction, full record writes, cleanup, and migration | Open | Gather Scratchpad/Todo evidence before scoped keys, typed records, documents, or collection transactions |
| TODO-F005 | Boundary confirmation | State imports cannot enumerate orphan `todo/<id>/...` keys outside `todos-order` | Deferred | Document the integrity boundary; Todo does not authorize enumeration |
| TODO-F006 | Application convention | Explicit projection diffing is substantial and an initial all-model row lookup exhausted turn fuel at capacity | Addressed locally; architectural question open | Limit lookup to the five-row projection; retain explicit patches while measuring size/convergence |
| TODO-F007 | Boundary confirmation | Explicit patches can now be checked against a fresh read-only reconstruction after every commit | Addressed at Gate B | All five scenarios pass `youth test --verify-view-convergence`; no automatic SDK diffing authorized |
| TODO-F008 | Tooling defect | Runtime test failures originally omitted the stable error category, obscuring `FuelExhausted` | Addressed | `.youth-test` diagnostics include the stable category without exposing raw guest errors |
| TODO-F009 | Platform discovery | A five-row page replacement expands to many primitive create/detach/delete/attach patches | Open evidence | Keep measuring patch count and turn cost; do not add a list node solely to compress one app |
| TODO-F010 | Boundary confirmation | Todo remains free of WIT bindings, numeric IDs, revisions, acknowledgements, raw patches, and export plumbing | Confirmed | The DP0 SDK boundary survived dynamic collections and migration |
| TODO-F011 | Tooling defect | Windows converted vendored WIT line endings during checkout, so its exact-byte contract hash differed | Addressed | Pin `*.wit` to LF with `.gitattributes`; contract hashing remains exact and does not hide byte changes |

## Gate A blocker evidence

The minimum failing sequence was Add → Delete Task 1 → Add Task 2. The Timer
SDK revision could not construct a node or command identity from a runtime
`TaskId`: `NodeKey::new` and `CommandKey::new` required `&'static str`. Even if
an ID were available, `Update` exposed no structural operation. Reaching for
generated WIT patches or numeric IDs would have breached the SDK boundary.

No protocol blocker was found. `youth:app@0.0.5` already carried create,
delete, insert-child, remove-child, and move-child. Gate B therefore added only
typed derived identities and strict SDK expansion over those existing patches.

## Gate C application evidence

Commands exercised: Add, Toggle/Reopen, Delete, Move Up/Down, Clear Completed,
three filters, and Previous/Next. Five semantic scenarios cover persistence,
restart, v1→v2 migration, stable non-reused IDs, filtering, ordering, paging,
focus retention/clearing, and the 64-item capacity. Every scenario runs through
the real headless runtime with convergence verification enabled.

The capacity scenario exposed TODO-F006. `child_element` originally scanned all
64 durable items separately for each of five inserted rows. By page five this
crossed the host's fixed 20,000,000 handle-fuel budget. Restricting resolution
to `TodoSession::visible_ids` made the work bounded by five and all scenarios
passed without increasing host limits. This was an application algorithm defect,
not evidence for more protocol or policy.

The pure `presentation_diff` deliberately removes absent rows, moves retained
rows, inserts new rows, then updates semantic properties. That surface is larger
than rebuilding a tree, and it requires the app to know which fields affect
which nodes. The convergence checker removes silent divergence risk in tests,
but the authoring repetition remains genuine evidence for future SDK diffing.
Todo alone does not decide between explicit patches, SDK tree diffing, or a
reactive model.

The first release matrix exposed TODO-F011: the WIT snapshot was authored with
LF bytes, but Windows Git checkout materialized CRLF bytes and correctly failed
the exact lock hash. The repository now declares LF as the checkout contract
for `*.wit`. Youth still hashes the exact inspectable snapshot; neither the CLI
nor CI normalizes away a real contract-byte change.

## Layer conclusions

- What could not be expressed: derived identities and structural updates in the
  published Timer SDK; both were solvable in the SDK with no WIT change.
- What felt repetitive: explicit state keys/migration and explicit presentation
  diff/property bookkeeping.
- What leaked WIT details: none in final application source.
- What required host policy: identity-global collision rejection, transactional
  patch validation, focus reconciliation, fuel, and read-only convergence.
- What protocol addition was unavoidable: none.
- What remains an SDK/application concern: derived-key ergonomics, structural
  expansion, session convention, persistence codec, projection diffing, and
  bounded lookup strategy.
