# Task 5 report: Deviations 42 and 43 name themselves

## Changes

* `Loud::fork_child` and `Loud::named_semaphore` (`rust/crates/rexx-exec/src/lib.rs`) build their message with no owner, as `object_name_of_a_value` does, and are `#[cold]`.
* Message texts, before and after:
  * `fork_child`: `routine "SYSFORK", whose forked child cannot continue without the timer thread and the joining main thread, is not implemented (Phase 9)` becomes `... is not implemented (Deviation 42)`.
  * `named_semaphore`: `routine "SYSOPENEVENTSEM" with a named semaphore, a POSIX sem_open semaphore shared between processes, is not implemented (Phase 9)` becomes `... is not implemented (Deviation 43)`.
* `rust/corpus/refusal-dispositions.tsv`: one `DEVIATION` row each, in alphabetical position.
* `rust/corpus/refusal-sites.tsv`: re-derived (line shifts only).
* Pinned texts updated in `rust/crates/rexx-exec/src/run/tests/directives.rs` (the only place they were pinned).
* Scanner test `the_scanner_reads_each_owner_form`: no constructor other than `native_method` still builds a phase-valued `Owner` in `src/` (the remaining `Phase10` uses pass an `Owner` parameter). `native_method`'s existing assertion in that test still covers the `Owner` value form, so the `named_semaphore` assertion now pins `Owner::None` and a matching `fork_child` assertion was added. Task 7, which removes `native_method`, must find a new value-form witness.

## Checks (all from `rust/`, `memcap 8G`, `-j 4`, `CARGO_INCREMENTAL=0`)

* `cargo fmt --all --check`: exit 0
* `cargo clippy -p rexx-exec --all-targets -- -D warnings`: exit 0
* `cargo test -p rexx-exec --lib`: exit 0, 1152 passed
* `refusal_sites` (with `REXX_REFUSAL_SITES_REFRESH=1`, then plain), `refusal_dispositions`, `owners`, `closed_phases`, `phase_ten_reasons`, `representation_refusals`: each exit 0
* `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus`: exit 0, 29 passed, 1 ignored
* `grep -rli 'sysfork\|SysOpenEventSem\|SysOpenMutexSem' ootest/ooRexx`: no matches (grep exit 1), so no ooTest test calls these routines.
