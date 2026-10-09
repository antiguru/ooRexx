# Refusals that still name Phase 5, which is closed

Phase 8's close (Task 9, ruling 1) relabelled `Loud::native_method` to Phase 9 and left the rest,
since `rust/crates/rexx-exec/tests/closed_phases.rs` records Phase 5 as a debt deliberately absent
from `CLOSED`. Found by, from the worktree root:

    git grep -n '"Phase 5"' -- rust/crates | /bin/grep -a /src/ \
      | /bin/grep -avE '^[^:]+:[0-9]+:[[:space:]]*//'

At 14636fe6a that names, in `rust/crates/rexx-exec/src/`: `lib.rs`'s `Loud::receiver_class`,
`operator_operand`, `object_position`, `method_from_source`, `object_method`, `method_body`,
`library_source`, `required_source`, `setup_method`, `expose_receiver` and
`use_local_in_a_method`; `lib.rs`'s `instruction_owner` arm for `InstructionKind::Options`;
`environment.rs:363`'s owed entries; and `redirect.rs:644` and `run.rs:3589` (`.STREAM`).

Relabelling them is one decision per owner, not a text edit: each needs the phase that owes the
work (most are Phase 9's core conformance), the tests pinning their text (`dispatch/tests.rs`,
`eval/object_operand_tests.rs`, `run/tests/directives.rs`, `tests/spike.rs`), the derived tables
that record crate output (`corpus/introspection-arity.tsv` is refreshed by
`REXX_INTROSPECTION_ARITY_REFRESH=1`), and `"Phase 5"` added to `closed_phases.rs`'s `CLOSED`.

## Closed 2026-10-09 by Phase 6.1 Task 7

Every constructor listed is deleted (Tasks 2-5) or has owner `None` with a row in
`rust/corpus/refusal-dispositions.tsv`; `"Phase 5"` is in `closed_phases.rs`'s `CLOSED`, and the
scan covers the owner tables in `tests/`.
