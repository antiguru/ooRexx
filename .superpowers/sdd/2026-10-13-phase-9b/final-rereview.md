# Phase 9b final fix re-review

Range 43a39ab01..6d7180b21, read from the diff file. Checks ran in a git-archive copy of 6d7180b21
with its own CARGO_TARGET_DIR (deleted afterwards), under memcap 4G. Nothing run on the oracle.

## Verdict: approved, no open findings

- I1: resolved. All three resolutions in stems.rs (stem_argument, sort_target whole name, sort_target prefix) push_temp.
- M1: resolved. Both new tests pass at HEAD and go red when their root is deleted.
- M2: resolved. SysCls passes cwd_bytes as OsStr. No lossy conversion remains in builtin/rexxutil non-test code.
  The DIRECTORY() and ADDRESS sites are recorded in the queued lossy-sites file, with install.rs:1905 added.
- M3: resolved. Entry 48 and the pin exist. Line refs match (:486 in this checkout, :569 at cb9563364; both lines read `name = getenv("HOME")`).
- M4: resolved. Entry 37 and entry 45 name existing tests.
- M6: resolved. The listed semicolons, the ` -- `, and the false set_cwd sentence are gone. Added comment/prose lines contain no
  em-dashes, structuring semicolons or future-tense mechanism text (the only added semicolons are inside Rexx source in test strings).
- M5 and N1 untouched, as ruled.

## Mutants (one at a time, tests run with --no-fail-fast)

| deletion | red test |
|---|---|
| stems.rs:49 stem_argument push | a_stem_named_by_its_variable_outlives_the_variable |
| stems.rs:544 sort whole-name push | same |
| stems.rs:557 sort prefix push | same |
| stems.rs:731 dump `variables` push | a_dump_keeps_the_values_of_variables_a_string_method_drops (panic dispatch.rs:1710) |
| messages.rs:250 array push | a_fresh_make_array_answer_outlives_its_values_string_methods |
| process.rs SysCls reverted to cwd_text | sys_cls_runs_in_a_directory_whose_name_is_not_utf8 |

The other tests stayed green in each run. All five new tests pass at HEAD.

## Minor observations (not blocking)

- Entry 47's oracle table (SIGSEGV for copy/delete/insert at 200000 allocations) is the fixer's measurement. I did not and
  must not re-run it. It differs from the final review's 40.922, which the entry says openly.
- 7d4743be9's message omits M2, which it contains (disclosed in the fix report).
- The stem_dropped_by_name case file has five cases but only one test asserts its normal==stressed equality plus the
  recorded block, so the pin is the crate's own answer by design.
