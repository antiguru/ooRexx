# Phase 9d Task 8.7c report: variables and streams residuals

Status: DONE_WITH_CONCERNS. Base `97eee8a9a` (dispatch commit `34a25ee6b` on top). Code `a33114804`,
`e335a9162` and `e620097f0` (the perf round); records, closings and inventory rows in the commit after,
with this report; the workspace result in the commit after that. The closing lines and inventory rows
name `e620097f0`, the last code commit. Records: `docs/superpowers/records/2026-10-18-phase-9d/t8.7c/`
(`three.sh` the runner: `o` oracle, `b` base `rexx-base-97eee8a9a`, `h` `rexx-head-e620097f0`, each
probe directory's outputs under `out/`; `witnesses/out/` the three-way runs of the new corpus programs
from their scratch copies; `suite/`, `checks-*.txt`, `callgrind.txt`).

## What changed, per item

**2026-10-27-varref-operator-traceback-line.md (closed).** Rule: an operator is a message, and a
VariableReference forwards a message it does not know to the referenced value, so the failure is
inside that value's operator method. `is_forwarding_receiver` (`eval.rs`, was `is_stem_receiver`)
answers for a reference whose chain ends at a string or number value or an unset variable, so
`blame_forwarded_operator` adds the `Compiled method "<op>" with scope "String"` line on every
arithmetic, prefix, logical, comparison and loop-header failure. Probes `probes/varref/v1`, `v3`, `v4`
(o~b differ, o~h equal). Witness `corpus/lang/variable_reference_operator_traceback.rex`.

**2026-10-27-stem-array-default-string-value.md (closed).** Rule: rexxref intro.xml:2653-2656, "the
STRING method will return the Stem object's default value's string representation", and TRACE renders
a non-string by its STRING (instrc.xml:3697-3699). `string_value_text` (`value.rs`) renders a stem as
its default's `string_value_text` (`stem_string_value_text`, the chain of defaults and references);
`Object~string` of a Stem subclass instance takes the same. `t.~string`, `u~string` and `trace r; x =
t.` answer `an Array` (`probes/stemarr/a1.rex`). `say t.` still joins the items here: Deviation
candidate 1. Witness `corpus/lang/stem_default_string_rendering.rex`.

**2026-10-22-lineout-four-args-error.md (closed).** Rule: rexxref does not order the two errors; the
oracle's `NativeActivation::processArguments` converts each declared argument before it counts the
rest (`NativeActivation.cpp:682`). `stream_charin`, `stream_charout`, `stream_linein` and
`stream_lineout` are `Arity::Counted` and call `declared_only` after their conversions
(`dispatch/stream.rs`, `dispatch/native.rs`). Probe `probes/lineout4/l2.rex`. Witness
`corpus/lang/stream_method_extra_arguments.rex`.

**2026-10-28-internal-rendering-defaultname.md (closed).** Rule: rexxref fundclasses.xml, Object
`string`/`objectName`/`defaultName`: the string representation is the objectName, which is the
defaultName where none is set. The oracle's `RexxObject::stringValue` sends `OBJECTNAME`
(`ObjectClass.cpp:1157`), whose primitive sends `DEFAULTNAME` (`:1713`). `string_value_text` now sends
`OBJECTNAME` to an instance or class whose `OBJECTNAME` is a method of its own, else `DEFAULTNAME` under
the primitive `OBJECTNAME`'s level where that is one of its own and no name is set
(`sent_string_value`); `default_string_value_text` is the unsent rendering the primitives
`OBJECTNAME`, `DEFAULTNAME` and `STRING` answer with. `Object~string` and `~objectName` push their
levels where the send reaches Rexx code, and `Array~makeString`/`~toString` run as leveled primitives
(`RENDERING_METHODS`, `dispatch.rs`). `string_value_text` reaches the sends through an opaque function
pointer (Perf). Probes `probes/dn/dn4`-`dn7` (dn7 differs in the count of sends
under trace: Concerns 2). Witness `corpus/lang/internal_rendering_default_name.rex`.

**2026-10-28-trace-object-renderings.md (closed).** Rule: TRACE renders a non-string result by its
string representation (instrc.xml:3697-3699), and `VariableReferenceOp::evaluate` traces the
reference's `getName()` (`VariableReferenceOp.cpp:116`). PARSE VALUE's `>K>` renders the value by
`string_value_text` (`parse_template.rs`); `>O> ">"` names the reference's variable (`eval.rs`).
Probes `probes/traceobj/trace-parse-value-a14.rex`, `s25.rex`, `r1.rex`. Witness
`corpus/lang/trace_reference_and_parse_value.rex`.

**2026-10-28-primitives-without-frame.md (closed).** The set, from the oracle: the collection
primitives that compare or hash an argument through `equalValue`, `isEqual` or `hash`
(`ArrayClass.cpp:2107`, `ListContents.cpp:758`/`:793`, `HashContents.hpp:430`/`:437`,
`StemClass.cpp:1020`, `DirectoryClass.cpp:417`), each of which runs under its `InternalActivationFrame`.
A new `NativeBody::Compared` (`dispatch.rs`, rows in `COMPARED_METHODS`, converted by method after the
table is built) runs such a primitive on the ordinary native path, as its own level where an argument
is not of a base class (`may_run_rexx`); `Table~at`/`~[]`, whose row `Directory` shares, level inside
`hash::hash_at` (`[]` is now its own function so the level has its name); the rows already leveled by
Task 8.7b (Array `HASITEM`, the hash `PUT`/`[]=`) are unchanged. Probe `probes/primframe/f4.rex`:
every frame now agrees; the rows that still differ are which messages the crate sends (Queue candidate
1). Witness `corpus/lang/collection_primitive_compare_frames.rex` (the agreeing rows).

**2026-10-29-notready-delay-across-internal-call.md (closed).** Rule: rexxref condtra.xml, Notes: the
state (ON, OFF or DELAY) of each trap is saved on entry to a subroutine, so an internal routine
inherits its caller's DELAY; Deviation 75 ignores a NOTREADY raised while delayed.
`notready_trap_delayed_at` (`run/condition.rs`) counts a condition queued at any activation the
internal-call chain from the trapping one reaches. n4 and n7 run the handler once (`after a, m = 1`)
at head and twice at base (`probes/notready/out/*.b.out`, `*.h.out`; never run on the oracle).
Witness: two new cases in `tests/delayed_trap_cases/crate`.

**2026-10-28-stream-query-residuals.md (not closed: part 3 is Deviation candidate 5).**
* Part 1 (fixed). Rule: a console redirected from a regular file is a persistent stream open for
  reading (rexxref funct.xml, STREAM QUERY SIZE, POSITION, DATETIME, SEEK). `.STDIN` and an opened
  program-made STDIN answer from the file standard input reads (`Interp::input_file`, `input.rs`;
  `standard_input_file`, `seek_standard_input`, `query_size`, `query_time`, `query_position`,
  `dispatch/stream.rs`; `position`'s move is now `seek_open`, shared). Probes
  `probes/stdinq/q2.rex`, `q6.rex` (stdin `in.txt`): o~h equal. `q5.rex` still differs on `.STDOUT`'s
  queries (Queue candidate 3) and on a write-position or zero SEEK, which Deviation 59 refuses. Witness
  `input_oracle::a_file_console_answers_queries_and_seeks` (gated; red at base, `q6.rex.b.out`).
* Part 2 (ruled, unchanged). LINES and CHARS on a stream opened WRITE answer 0 under Deviation 55
  (`probes/lineswo/w1.rex`).
* Part 3: Deviation candidate 5.

**2026-10-28-append-stream-positioned-writes.md (not closed: Deviation candidate 3).**

**2026-10-28-hash-lookup-identity-first.md (not closed: Deviation candidate 2).**

**2026-10-27-stem-default-arithmetic-nostring.md (not closed: Deviation candidate 1).**

**2026-10-29-debug-input-traps-off.md (not closed: Deviation candidate 4).**

**gate:phase-9c-gate.md:139 (closed: probe agrees).** `2026-10-16-stream-command-open-missing-file.md`
was closed by Task 6.4; `probes/scopen/scopen.rex` agrees at base and head (`ERROR:2`, `rc ok`).

**gate:phase-9a-gate.md:485 (closed: probe agrees).** `samples` `TEST_MAKESTRING`, `scclient`
`TEST_04_SCCLIENT.REX` and `sfclient` `TEST_04_SFCLIENT.REX` pass on both sides in `suite/tests.tsv`
(and in `t8.7b/fix-r3/suite/tests.tsv`).

## Deviation candidates (not written)

1. **A stem in a required-string position.** Rule: a string value is `request('STRING')` (rexxref
   provide.xml:758), and a Stem's `request` forwards every class but ARRAY to its default
   (collclasses.xml, Stem `request`), so a stem's string value is its default's, and NOSTRING is
   raised only where that has none. Here `s. = 7; say s. ''` is `7`, and `t.` with an array default
   joins the items. ORACLE: `StemClass` has no `primitiveMakeString`, so `requestString` on any stem
   takes `stringValue` and raises NOSTRING naming the stem (`ObjectClass.cpp:1240`-`:1247`): under
   `SIGNAL ON NOSTRING`, `say s. ''`, `length(s.)`, `'<'||s.` and `'x' = s.` with `s. = 7` all trap
   `NOSTRING |7| Stem`, and `say t.` prints `an Array`; in an arithmetic operand and a comparison's
   right operand with a default that has no string value it forwards and names the default (`1 + v.`
   `|NSTR| NSTR`), where this crate names the stem (`|a NSTR| Stem`) as Deviation 72 does for a
   reference to a stem (`probes/stemnostr/s1.rex`, `s2.rex`). This crate is not uniform either: `say
   v. ''` (the stem as a blank concatenation's left operand) names the default. Candidate: a stem
   names itself in every position (Deviation 72's stem sentence extended to a plain stem, and the
   concatenation receiver made to agree), or names its default in every position.
2. **A hash collection's lookup of the identical index.** Rule: "Index equality is determined by
   using the `==` method of index" (collclasses.xml, Table `at`, `hasIndex`, `put`, `remove` and the
   rest; items likewise), which this crate follows: `t[k] = 2` after `t[k] = 1` sends `k~'=='(k)`.
   ORACLE: `equalValue` compares identity first (`ObjectClass.hpp:369`, from `HashContents.hpp:430`,
   `ListContents.cpp:758`, `ArrayClass.cpp:2107`), so the index's own `==` is never sent for itself
   (`probes/hashid/hash-identity-first-idq.rex`: `== called` twice here, never there; the `other
   object` row agrees). Candidate: compare identity first (`==` treated as reflexive, which also takes
   the send off a lookup's hit), or keep the rule.
3. **Positioned writes on an APPEND stream.** Rule: APPEND "positions the write pointer at the end of
   the stream ... The write pointer cannot be moved anywhere within the extent of the file as it
   existed when the file was opened" (funct.xml:4307-4312), so a write SEEK, a CHAROUT start or a
   LINEOUT line within that extent is refused (0 and NOTREADY, as Deviation 59's invalid SEEK; the
   residual count for CHAROUT). Neither interpreter enforces it: `seek('=1 write char')` answers `1` on
   both. The oracle then keeps the logical position (`pos3 4`) while its writes land at the end;
   this crate reports the end (`pos3 17`); `lineout('L4', 1)` is NOTREADY here under Deviation 55 (an
   APPEND open is write-only) and writes on the oracle (`probes/append/a1.rex`). Candidate: enforce
   the extent rule for SEEK and the CHAROUT start, or license both answers.
4. **A condition raised while debug input is read.** Rule: "While user input is processed during
   interactive tracing, all condition traps are temporarily set OFF" (condtra.xml:489-494, Note 4),
   so every condition the `.DEBUGINPUT` read raises takes its default action. This crate drops one
   only where its trap is delayed (Deviation 75) and delivers the rest: `dp3`'s `h 2 from input`, dp2's
   first read in main (`t8.7b/fix-r3/dp2`, `dp3`). ORACLE: `debugPause` is set only around
   `debugInterpret` (`RexxActivation.cpp:2789`), so the read's conditions reach their traps; not run
   (the delayed shape is `oracle-crashes.txt` entries 64-67's). Candidate: extend Deviation 75's
   sentence to every condition the read raises (the paused activation's debug-pause flag set across
   the read, so `trap_for` answers none but SYNTAX; the debug case's expected block changes to no
   handler run), or keep the delayed-only drop.
5. **A SEEK of an unopened transient stream.** Rule: positioning a transient stream is invalid,
   93.958. ORACLE: `stream_position` tests `transient` before its implicit open sets it
   (`StreamNative.cpp:2694`), so the first SEEK of `.stream~new('STDIN')` with standard input
   `/dev/null` answers `1` and `READY`; after `open('read')` it is 93.958 on both
   (`probes/stdinq/q4.rex`). This crate raises 93.958 either way. Candidate: license the difference.
6. **TRACE and Stem STRING renderings of an object overriding STRING.** Rule: TRACE sends STRING
   (instrc.xml:3697-3699) and a Stem's STRING is its default's string representation (intro.xml:2656),
   so a default or traced value whose class overrides `STRING` renders as that answer. Both
   interpreters render by `stringValue`, the `OBJECTNAME`: `v. = .nstr~new; say v.~string` and `trace
   i; x = v.` show `a NSTR` on both where the rule gives `NSTR` (`probes/stemnostr/s2.rex`,
   `probes/stemarr/b.rex`). Unchanged here. Candidate: follow the rule, or record the
   shared answer.

## New oracle defects (for defects.json; not edited)

1. Any stem in a required-string position raises NOSTRING naming the stem, whatever its default:
   `s. = 7; signal on nostring; say s. ''` traps `NOSTRING` with description `7`
   (`probes/stemnostr/s2.rex`; candidate 1).
2. The first SEEK of an unopened transient stream answers `1 READY` (`probes/stdinq/q4.rex`;
   candidate 5).

## Queue candidates

1. **Which messages a hash collection sends.** `Table~remove`, `Set~remove` and `Set~removeItem` send
   the index's `HASHCODE` twice where the oracle sends it once; `Bag~hasItem` and `Relation~allAt`,
   `~hasItem`, `~items` and `~removeItem` scan with `==` where the oracle hashes the index and finds
   an empty bucket (`probes/primframe/f4.rex`, the rows outside the witness).
2. **TRACE renders an assignment's value once for its `>>>` and `>=>` lines**, where the oracle
   renders each line, so a `DEFAULTNAME` of the object's own runs twice here and three times there
   for `x = o` under `trace i` (`probes/dn/dn7.rex`).
3. **`.STDOUT`'s QUERY SIZE and POSITION when standard output is a regular file** answer the null
   string here and the file's size and position on the oracle (`probes/stdinq/q5.rex`, `out size 34
   1`).

## Witnesses

New `corpus/lang` programs, in `phase-9.txt`, each with its `sourceline_oracle` file generated by the
test's driver from a scratch copy: `variable_reference_operator_traceback`,
`stem_default_string_rendering`, `stream_method_extra_arguments`, `trace_reference_and_parse_value`,
`internal_rendering_default_name`, `collection_primitive_compare_frames`. Each differs from the oracle
at base and agrees at head on stdout, stderr and rc (`witnesses/out/`, run from the scratch copies;
at head also by the corpus gate). I read each oracle stdout for the paths it claims: each failing
operator's `Compiled method` line and the agreeing `say s + 0` and `t = 1` rows; `an Array` for the
stem, the subclass and the stem-default stem and `7`; each 88.907 naming argument 1 or 2 and each
88.922; the `>K>` `an Array` line and each `>O> ">" => "V"`; `DNC`, `DNI`, `DN` and `ON` with their
`OBJECTNAME`/`STRING` frames and the named instance's `named`; each primitive's `METHOD` frame. Also
`input_oracle::a_file_console_answers_queries_and_seeks` and two `delayed_trap_cases` cases (above).

## Perf

`callgrind.txt`: `bench-programs/callgrind.sh -B 0.5 -j 6 -o /home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.7c/final2
base=.../rexx-base-97eee8a9a head=.../rexx-head-e620097f0`, every program, 3 rounds, exit 0, no OVER or
SPREAD flag; every `.vg` names one of the two binaries (99 each). Libc subtracted, every program is at
most +0.0404% (pingmsg); pingsem +0.0351%, startup +0.0199%, hashnew +0.0198%; the largest falls are
heapshape -0.0436% and dirread -0.0223%. Whole process at most +0.0323% (dirread). The base was built
from a `git archive` of `97eee8a9a` (`rust/`, `interpreter/`, `extensions/`) in its own target dir.

The full run at `a33114804` was stopped after its first round (`callgrind-a33114804-r1.txt`, from its
`.row` files): dirread +4.50%, parse +1.39%, rexxcps +0.33%, fibcall +0.27%, startup +0.24%. The causes,
each found by `cgdiff.py` and per-line callgrind costs on one-round reruns (`cg-9d/t8.7c/try2`-`try20`):
the `Compared` rows dispatched through `run_other` with `is_base_class` on every argument (dirread);
the row conversion's linear search over every native row at build (startup, +140k instructions); a
direct call from `string_value_text` to the sends, which moved register allocation in callers that
render only on a failure (`header_number`, `exec_parse`, `resume_call`; reverting `value.rs` alone
removed it, and calling through `std::hint::black_box` keeps it removed); a third caller of
`hash::store_at`, which stopped it being inlined (dirread); and a changed `rendered_into_parse_buffer`
signature (parse, fibcall).

## Commands

All from `rust/`, `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=/tmp/claude-1000/t87c-target`, at
`e620097f0`; outputs in `t8.7c/`.

| command | result |
|---|---|
| `cargo fmt --all --check` | 0 |
| `memcap 16G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings` (`checks-clippy.txt`) | 0 |
| `memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-exec --lib --test owners --test closed_phases --test phase_ten_reasons --test representation_refusals --test refusal_sites --test refusal_dispositions --test method_bodies --test coverage --test signals` (`checks-touched.txt`) | 0; lib 1207 passed |
| `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-exec --test input_oracle --test stream_handle --test stream_model --test standard_stream_reopen --test stdin_contention --test collection_arity --test trace_oracle --test trace_indent --test licensed_divergences --test native_entries --test dispatch_seam` (`checks-streams.txt`) | 0; input_oracle 27 passed, none skipped |
| `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus` (`checks-corpus.txt`) | 101: the one mismatch the known `rexxutil_file_date_time.rex`. The run at `a33114804` also had `command_redirected_target_waits_elsewhere.rex` (`guard 0 3 waited 0` against the oracle's `waited 1`), the known redirected-target timing flake |
| `memcap 16G cargo test -j 4 --release --no-fail-fast -p rexx-parse --test sourceline_oracle --test errors --test tiling --test variants` (`checks-parse.txt`) | 0 |
| `memcap 16G cargo test -j 4 --release -p rexx-exec --test collect_stress` (`checks-collect-stress.txt`) | 0, 74 passed |
| `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=.../t8.7c/suite/table.tsv memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite` (`checks-suite.txt`) | 0; `tests.tsv`, `table.tsv`, `verdicts.tsv`, `oracle-differences.tsv` identical to `t8.7b/fix-r3/suite/` (also at `a33114804`) |

## Concerns

1. **A failing `OBJECTNAME` or `DEFAULTNAME` send inside a rendering is dropped**: `string_value_text`
   returns bytes, so a method of the object's own that raises renders the default name here, where
   the oracle raises from the trace line, message or join. No probe reaches it.
2. Queue candidate 2 is visible only through a side-effecting `DEFAULTNAME` under trace.
3. Four items stop at Deviation candidates and stay open; part 1 of the stream-query item is fixed but
   the item stays open for candidate 5.
