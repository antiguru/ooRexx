# The SELECT CASE value stays on the activation after its construct ends

Found by Phase 6.1 Task 1 (report concern 2; deferred minor "current_case_text is never cleared at construct end"). Queued by Phase 6.1 Task 12 (2026-10-10).

The case value is set when each `SELECT CASE` opens and never cleared. No program found reads a stale value: an absorbed WHEN follows its own construct's opening with nothing in between that runs in the same activation, and a typed debug line saves and restores it (Task 1 fix round 1). No probe shows a divergence; the item is the unenforced "for the construct's duration" property, which a debug assertion at construct end could pin.

Suspected site: `Activation`'s `current_case_text` (`activation.rs`), set by `open_select_case`.
