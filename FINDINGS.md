# Todo Findings

This is the canonical evidence record for Youth Todo. Findings are evidence,
not automatic authorization for platform features.

## Open findings

| ID | Category | Observation | Status | Implication |
| --- | --- | --- | --- | --- |
| TODO-F001 | Platform discovery | Published `NodeKey` and `CommandKey` accept only `&'static str`; Task 1/Task 2 identities cannot be derived from durable runtime IDs | Open at Gate A | A bounded SDK-derived identity must be proven before any protocol change |
| TODO-F002 | Platform discovery | Published `Update` exposes only text/countdown/label/enabled property changes although protocol `0.0.5` already carries structural patches | Open at Gate A | Expose the smallest explicit SDK subtree operations demonstrated by Add/Delete |
| TODO-F003 | Application convention | Filter and page are projection state, but `Application` has no instance/session object | Open | Use application-owned process-local session state; do not persist navigation merely for convenience |
| TODO-F004 | Platform discovery | Explicit per-item keys require an order index, repeated key construction, cleanup, and custom migration | Open | Collect evidence before choosing scopes, typed records, documents, or collection transactions |
| TODO-F005 | Boundary confirmation | State imports cannot enumerate keys, so orphan per-item records outside `todos-order` cannot be detected | Deferred | Do not add enumeration during Todo; document the integrity boundary |

## Gate A evidence

The minimum failing sequence is Add → Delete Task 1 → Add Task 2. The published
SDK revision in `Youth.lock` cannot construct a node or command identity from a
runtime `TaskId`: `NodeKey::new` and `CommandKey::new` require `&'static str`.
Even if an ID were available, `Update` has no create, insert, remove, delete, or
move operation. Using the WIT-generated patch types or numeric IDs would breach
the SDK boundary proven by Calculator and Timer, so Gate A stops here.

No protocol blocker has been found: `youth:app@0.0.5` already contains create,
delete, insert-child, remove-child, and move-child patches.
