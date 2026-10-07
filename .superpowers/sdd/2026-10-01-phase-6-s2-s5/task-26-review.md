# Task 26 review: the Phase 6 performance gate

Range `c484f4516..1bd4e7913`. Read-only; scratch `/tmp/claude-1000/p6-t26-rv/` (deleted after).

**Verdict: PASS WITH ISSUES.** The three code rounds are behaviour-preserving and each change
is pinned by the existing suite. Every Ir figure in the record matches a committed table. The
findings are about the record's prose and wall clock, and none changes a verdict.

## Part 1: code rounds (`ba8f7c581`, `634f591a8`, `05aac0c58`)

What was read: `git diff c484f4516 05aac0c58 -- rust/`. No `rust/` change after `05aac0c58`.
Nothing compares a name `Rc` by pointer (`grep -rn "ptr_eq\|get_mut(\|make_mut"` in
`rexx-exec/src`: no name site). `Activity::invocation_name` shares only on byte equality. The kept
`EXPOSE` list is `clear()`ed before it is stored, so it carries capacity only. It is stored only on
the non-`REPLY` path of `release_method_activation` and is taken only when the activation's list
has capacity 0.

Probes, oracle against `rexx-run` built at `1bd4e7913` (`/tmp/claude-1000/p6-t26-rv/probes/`):
- `names.rex`: alternating and repeated CALL names in 40.3 messages, recursion and mutual
  recursion through `.context~name`, FORWARD own-name across alternating sends.
- `expose.rex`: different exposed names in successive methods, a 41.1 raised mid-method, an
  `EXPOSE (sel)` that binds a name and then raises 31.2, recursive sends that expose, a cross-object
  send, and an internal label call that inherits the exposure.
- `reply.rex`: `REPLY` and then a send that exposes in the continuation, two `~start`ed unguarded
  methods exposing different names and sleeping (activity switches), repeated CALL names.
- `trace.rex`: `::options trace i`, with `>I>`/`<I<` method and routine names across alternating calls.

Each ran with the oracle wrapper from a fresh empty dir. stdout, stderr and rc are identical on all
four (`names` rc 163, the others rc 0). `reply.rex` ran 8 times per engine and gave one output
hash per engine, the same hash on both.

Mutation (`memcap 8G cargo test --release -p rexx-exec -j 2 --no-fail-fast -- --test-threads=2`
after a `--no-run` build; each mutant by Edit, restored by Edit, tree clean afterwards):

| mutant | site | passed / failed | catchers (examples) | probes |
|---|---|---|---|---|
| none | `1bd4e7913` | 1897 / 0, exit 0 | | |
| M1: `Some(last) =>` (name shared regardless of bytes) | `activity.rs` `invocation_name` | 1886 / 11 | `eval::tests::a_routine_returning_no_value_in_expression_form_raises_44_1`, `run::tests::scope::use_strict_arg_reports_method_errors_in_a_method_and_call_errors_outside`, `every_directive_options_program_answers_the_oracle` | `names`, `reply` differ |
| M2: `exposed.clear()` -> `truncate(usize::MAX)` (stale entries kept) | `dispatch.rs` `release_method_activation` | 1879 / 18 | `dispatch::tests::a_directory_subclass_keeps_the_instance` (88.914 instead of `K 1 5`), `the_l0_subset_passes_again_under_collect_on_every_allocation` | `expose`, `reply` differ |
| M3: `capacity() == 0` -> `< usize::MAX` (spare taken on every bind) | `run.rs` `bind_exposed` | 1884 / 13 | `corpus_differential` (`context_of_another_activity.rex`, `context_moved_by_reply.rex` did not finish), `no_row_started_diverging_or_stopped_answering` | `expose`, `reply` differ |

Round 1 is a type change (`Vec<u8>`/`Box<[u8]>` to `Rc<[u8]>`) with the same bytes at every site,
so no value mutation applies to it apart from M1's; M1 covers the name it carries.

No code finding.

## Part 2: the record

Checked by script against the committed tables: every row of `phase-6-perf.md` `## S2-S5 gate`'s
round 1 table (against `cg-table.txt`, `r1/cg-r1-table.txt`), its r3 table (against
`r3/cg-r3-table.txt`), and its wall table (against `wall/table.txt`). They match, including the
verdict column recomputed with `r1/verdict.sh`'s rule. The round 2 figures match `r2/verdict.tsv`,
the candidate table matches `experiments/exp{A,D,E,F,G}-table.txt` against r1, and the reproduction
offsets match the perfbase `cg-table.txt`. The "about 24 Ir per call" figure and "3 Ir more per
clause" also check out. Over-budget rows are stated as accepted under the 2026-10-07 ruling (perf
`### Verdict`, `phase-6-gate.md:1285-1287`). The reproduction offset is stated (`phase-6-perf.md:1333-1338`). The wall
load is quoted.

### R1: extcall's wall regression has no stated cause and no measured head
- Location: `phase-6-perf.md:1497`, `phase-6-gate.md:1295`.
- Scenario: the extension-call loop is a program named in spec section 7. In wall time, s1 is
  -3.23% and r1 is +14.85% against base, while its Ir is -8.80% (`wall/table.txt` row `extcall`,
  `cg-table.txt`). `head` (`c484f4516`) is not in the wall run, so the record does not show where
  the rise came from. P19 carries "wall excess from layout" as recorded, but nothing shows that
  this excess is layout.
- Command: `grep -E "^extcall" docs/superpowers/records/2026-10-01-phase-6-s2-s5/perf/wall/table.txt`
  gives `extcall 0.929 0.899 1.067 1.076 0.930 0.530 -3.23 +14.85 +15.82 +0.11 -42.95`.
- Severity: Low.
- Fix: say in the wall subsection that `extcall`'s rise from s1 is not attributed (head was not
  measured). Or put it to the lead as a Phase 6.1 candidate.

### R2: the identical-binary control is itself over the wall bar
- Location: `phase-6-perf.md:1465-1499`.
- Scenario: `base2` (a copy of base) shows `decloop` +6.48%, which is over both ±4% and that
  program's zero-work band (+6.10%). The record does not mention it, and it bounds how every
  over-bar wall row can be read.
- Command: `grep -E "^decloop" .../perf/wall/table.txt` gives `... 0.263 ... +6.48 -`.
- Severity: Low.
- Fix: one sentence stating the control's largest delta.

### R3: "round 1 rows" means callgrind repetition r1, in a section where round 1 is `ba8f7c581`
- Location: `phase-6-perf.md:1385`, `:1452`, `:1504`.
- Scenario: `r3/summary.tsv`'s third column is the repetition (`r1`..`r3`). "(`r3/summary.tsv`,
  round 1 rows)" therefore reads as the round 1 commit's rows, which that file does not have
  (`awk -F'\t' '{print $1}' r3/summary.tsv | sort -u`: base, r3, s1).
- Severity: Low.
- Fix: "repetition `r1` rows".

### R4: libc-included `dispatch` at round 1 is +2.19%, not +2.20%
- Location: `phase-6-perf.md:1384`; `perf/diagnosis.md:100`.
- Command: `python3 -c "print((23337519525-22836264725)/22836264725*100)"` (the `r1/summary.tsv`
  `dispatch` repetition r1 whole counts) gives `2.19499469828466`.
- Severity: Low.
- Fix: +2.19%.

### R5: expJ does not have "the same Ir as the round's run"
- Location: `phase-6-perf.md:1409`.
- Scenario: expJ against `r3/cg-r3-table.txt`: no program is equal; the differences run from
  `rexxcps` -1283 to `dispatch` +266 Ir. That is the environment offset size, but "the same" is false.
- Command: `join` of `experiments/expJ-table.txt` column 3 with `r3/cg-r3-table.txt` column 4 on
  program; 0 rows equal.
- Severity: Low.
- Fix: "within 1.3 kIr of the round's run".

### R6: the reproduction cause is stated as established; the evidence shows only that it is possible
- Location: `phase-6-perf.md:1336-1337`.
- Scenario: `cgenv-table.txt` shows that one extra variable moves `startup` by +381 Ir, and
  `cgpath-table.txt` shows that the binary path moves nothing. No run reproduces the perfbase
  environment and gets 0 offset, so "The cause is the process environment" goes past the evidence.
  Also, `rexxcps` base varies by 17,462 Ir across the four runs in this record (`cg`, `cg-r1`,
  `cg-r2`, `cg-r3`: 17817334676..17817352138), against a band of 3,388. That does not touch the
  verdict (-0.16% against a +1.0% bar).
- Severity: Low.
- Fix: "attributed to the process environment", and state `rexxcps`'s cross-run spread beside the band.

### R7: the record does not say rounds 2 and 3 ran before the ruling was read
- Location: `phase-6-perf.md:1513-1516`, `phase-6-gate.md:1285-1287`.
- Scenario: the verdict quotes the ruling as "no further rounds" and lists rounds 2 and 3, which
  `progress.md:388` records as "run before the 'accept and record' message was read". A reader
  sees two rounds after a no-more-rounds ruling.
- Severity: Low.
- Fix: one clause saying they ran before the ruling was read (`progress.md:388`).

No forward-looking prose found. Counts in prose are quoted beside their source (`-r 5`, 1,719,390
calls in `diagnosis.md`).
