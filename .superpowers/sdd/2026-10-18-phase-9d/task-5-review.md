# Task 5 review: Deviations 42 and 43 name themselves

Reviewed `70a7a4965..b01fcb42a` (HEAD checked: b01fcb42a).

## Spec compliance: compliant

* lib.rs:536-560: both refusals build `format!` with no `Owner`, name their Deviation, carry `#[cold]`.
* refusal-dispositions.tsv: `fork_child` and `named_semaphore` are `DEVIATION	-	Deviation N: ...` rows in the same shape as `object_name_of_a_value`, in alphabetical position.
* refusal-sites.tsv: `refusal_sites` 5 passed, `refusal_dispositions` 3 passed. Diff is line shifts plus the two sites.
* Scanner test: `native_method` still asserts `Owner::Phase("Phase 9")`, so a phase-valued `Owner` is still proven. The two new assertions pin `Owner::None`.
* stderr, rc 120 on both:
  * `routine "SYSFORK", whose forked child cannot continue without the timer thread and the joining main thread, is not supported (Deviation 42)`
  * `routine "SYSOPENEVENTSEM" with a named semaphore, a POSIX sem_open semaphore shared between processes, is not supported (Deviation 43)`

## Findings

* Minor: task-5-report.md still quotes "is not implemented (Deviation 42/43)", stale after b01fcb42a.
* Minor: after Task 7 removes `native_method`, the scanner test loses its only phase-valued `Owner` witness (report already flags it).

## Task quality: Approved
