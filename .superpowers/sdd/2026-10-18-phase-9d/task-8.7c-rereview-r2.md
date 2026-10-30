# Phase 9d Task 8.7c re-review of fix round 3

Reviewer t87c-rr2. Code BASE `d6720f350` (code as at `8dd9a4ae0`), HEAD `3344571cc` (code as at
`cd070fd13`; `git diff --stat cd070fd13 3344571cc -- rust` is empty).

**Verdict: CHANGES REQUESTED** (Critical 0, Important 1, Minor 0). Items 3 to 6 are ADDRESSED. Items 1
and 2 are ADDRESSED for every shape the brief names, but the round's change to `to_text` causes the
Important finding R2-1: it reopens N2's untraced send through a reference and drops conditions at
sites that can carry them. Concern 2 is wrong: the drop can be reached by running a program.

## What I ran

* `git archive` trees of `3344571cc` and `d6720f350` (`rust interpreter extensions`, read-only
  symlinks to the checkout's `build ootest api docs samples testbinaries oodocs`). Each built with
  `CARGO_INCREMENTAL=0 memcap 8G cargo build --release -j 4 --locked --bin rexx` into
  `/tmp/claude-1000/t87c-rr2-target-<sha>`, exit 0. The head binary's `.text` is byte-identical to the
  pinned `cg-9d/t8.7c/rexx-head-cd070fd13` (`objcopy -O binary -j .text`, `cmp`).
* Probes via `review-probes-rr2/bin/r.sh`: `o` oracle under the standard wrapper, `b` base, `h` head,
  crate under `memcap 2G timeout -k 5 20`. Every run is in one fixed fresh directory, so the paths in
  stderr match. stdout, stderr and rc are kept separately under each probe directory's `out/`, and
  `bin/cmp.sh` compares them. Shapes 64 to 67 of `oracle-crashes.txt` were not run.
* Mutant builds, each from its own copy of the head tree:
  * `m`: `reference_text`'s `Err` arm prints `DROPPED` and a backtrace;
  * `n`: `reference_text` is `unsent_string_value_text` again, i.e. the `to_text`/`text_len`
    arms as at base;
  * `dep`: `eval.rs` mutants for the depth test (`bin/eval-depth-mutants.py`).
* From a head copy (`dep`, restored and `cmp`-identical to head's `eval.rs`):
  `REXX_CORPUS_GATE=1 CARGO_INCREMENTAL=0 timeout 590 memcap 12G cargo test --release -j 4 --locked
  --no-fail-fast -p rexx-exec --lib --test stream_handle --test corpus` exits 101 (`tests-summary.txt`):
  * lib: 1211 passed, 1 failed, `a_move_across_file_systems_copies_and_unlinks`. This failure is
    environmental: the target dir is on /tmp.
  * stream_handle: 8 passed, including `an_append_stream_refuses_positions_within_a_file_over_four_gib`.
  * corpus: "1236 of 1237 matching". The one mismatch is the known `lang/rexxutil_file_date_time.rex`.
* Callgrind (Perf).

## Brief items

| item | verdict |
|---|---|
| 1 N1 (reference renderings send, conditions propagate) | ADDRESSED for the named shapes; the `to_text` arm is R2-1 |
| 2 N2 (RAISE renders only for a printing trace line) | ADDRESSED for objects; through a reference, R2-1 reopens it |
| 3 LOSTDIGITS names the operand's string value | ADDRESSED |
| 4 M-a (extent over 4 GiB) | ADDRESSED |
| 5 M-b and concern 5 (comment, depth binding) | ADDRESSED |
| 6 Perf | ADDRESSED |

**1 / (a).** The review shapes `rr1/i1b/g.rex`, `i1d/k.rex`, `i1f/n.rex`, `i1g/p.rex` and `i1c/l.rex`
were rerun:
* `i1d/k`, `i1f/n`, `i1g/p` and `i1c/l`: oracle and head equal on all three descriptors, and base
  differs from the oracle on each.
* `i1b/g`: o~h differ on stdout only, by the single `>=>` send (concern 1, queue candidate 1 below).
* The three new corpus programs are o~h equal. Reading their stdout and stderr, each claimed path
  prints: the join, `~string`, `>V>`/`>>>`, `s1`/`s2`/`s3 98.900 boom.`, the untraced
  `s1 98.939`/`s5 93.900`/`USER` lines, and the traced `>K> "DESCRIPTION" => "NOISY"` followed by
  `s4 98.900`.
* `a/a1.rex` (a reference to an object with a `makeString`): `say r`, `||` and `length` run
  `makeString`, which is Deviation 72. The join, `r~string` and `>>>` send `DEFAULTNAME`, which is
  `r~string`'s STRING method on the value and Array's join through string values. The only o~h
  difference is Deviation 72's.

Concern 2 is refuted; see R2-1.

**2 / (b).** `n2/r1`-`r42` are `raise syntax (v)`, untraced and under `trace r` and `trace i`, with `v`:
* `'93.901'`, `93.9`, `' 93.901 '`, `'40'` or `'x'`;
* an object with a `makeString`, a noisy `DEFAULTNAME`, a raising `DEFAULTNAME`, or a `makeString`
  answering `.nil`;
* a reference to each of those objects;
* an Array, or `.nil`.

All 42 are o~h equal on stdout, stderr and rc except `r6`/`r20`/`r34`: a reference to a `makeString`
object. There the oracle sends `DEFAULTNAME` and answers 33.904. Head runs `makeString` and answers
93.901, which is Deviation 72's rule: the reference's string value is the value's
`request('STRING')`. So the code taken as `required_string_value` of the RC matches
RaiseInstruction.cpp:191-197 (`rc->requestString()`, 33.904 on `.nil`) for every non-reference shape,
traced and untraced. The `>K>` line is the oracle's in each traced case.

DESCRIPTION and ADDITIONAL over objects are o~h equal untraced (`i1f/n`, the corpus witness), except
the pre-existing description rendering, queue candidate 2. Through a reference, R2-1 applies.

**3.** The `ld/l1`-`l22` probes cover `+ - * // % ** = < >`, a unary operator, `abs`, `trunc`, `max`
and blank-padded strings, at DIGITS 5 against an object whose `makeString` answers `1234567890`,
directly, through a reference and blank-padded. Every right-operand and comparison position is o~h
equal, `LOSTDIGITS 1234567890`, where base said `a KN`. Two positions differ at base and head alike:
`1 ** k` (the 26.8 insert, `found "1234567890"` where the oracle sends and says `"DNK"`; the rule's
string value is head's) and `5~max(k)` (queue candidate 3).

**4 / (c).** The R3 rule holds over 4 GiB. `big/b.rex` (crate only) ran on sparse files of 4294967296
and 5368709120 bytes, opened WRITE APPEND. On head, `seek =1`, `=4294967295`, `=4294967296`,
`=size-1`, `=size`, `<1` and `-1` all answer `0 NOTREADY`, `charout('x', size)` answers `1 NOTREADY`,
and `=size+1`, `=size+2`, `<0` and `charout` at `size+1` move READY. After a reopen BOTH APPEND on the
grown file, `=size+1` (now inside the file) is refused and the new end is accepted. Base accepts
`=4294967296` and every position after it, `charout` at `size` included: the saturation M-a named.

The size is unchanged. At head, `assert!(size_of::<StreamState>() <= 232)` compiles and `< 232`
fails E0080, so the state is exactly 232 bytes. At base, an added `== 232` compiles, and its control
`!= 232` fails E0080. `within_append_extent` keeps a negative position refused (`<0`/`-1` above).

**5 / (d).** The comment names `eval_in_parse`. Mutants of the test
`a_parse_pattern_and_an_expression_share_the_eval_depth_limit` (`mutants/depth-*.txt`):

| mutant | result |
|---|---|
| A: `eval_in_parse` without `depth -= 1` | red (`tests.rs:1005`, the first half's rc) |
| B: `eval_in_parse` decrements before `eval_node` | red (`:1019`, the past-limit half answers rc 0) |
| F: `eval_in_parse` with no bookkeeping | red (`:1019`) |
| E: `eval` decrements before `eval_node` | red, and `eval_raises_11_1_exactly_one_term_past_max_eval_depth` too |
| C: `enter_eval_node` replaced by a bare `depth += 1` | green |

The test binds both copies' depth balance and their place in the depth count. Mutant C survives
because each nested node goes through `eval`, whose check fires at the same depth. The only effect
of skipping the check at the pattern's top node is that it is skipped there. That is not a finding.

**6 / (e).** The re-measure (`callgrind-arith-parse.txt`):
`bench-programs/callgrind.sh -B 0.5 -r 3 -j 6 -p "arith parse" -o .../cg-9d/t8.7c-rr2
base=.../rexx-base-97eee8a9a head=.../rexx-head-cd070fd13`. Status 0, and all 12 `.rc` files are 0.
Every `.vg` names one of the two pinned binaries, 6 each.
* arith: +0.0440% whole process, **+0.0464% libc subtracted**, spread 0.
* parse: +0.0372%, +0.0396% libc subtracted.

Both equal the report to the fourth decimal. In `fix3-final2`, every `.vg` names
`rexx-head-cd070fd13` or `rexx-base-97eee8a9a` (99 each), and all 198 `.rc` files are 0. Across the
33 rows of `fix-r3/callgrind.txt`, the largest libc-subtracted delta is arith's +0.0464 and the
largest spread is 0.0037 (define). Every program is within the +0.05% budget.

**(f).** Concern 1 is confirmed and pre-existing (`c1/c.rex`, `c1/d.rex`; base equals head). Under
`trace i`, the oracle sends `DEFAULTNAME` a third time for `>=>`, and with a third send that raises
it traps `98.900 boom.` where base and head print `>=>   W <= "ONCE"` and run on. Under `trace r`
both send once. The round did not cause it: queue candidate 1.

Concern 3 is refuted as a defect. `c3/*.rex` are a top-level `raise user zz` under CALL ON or SIGNAL
ON, with no option, `exit`, `return` or `exit 'E'`, and a `call r` whose `r` raises. The oracle, base
and head all end silently with rc 0, which is the documented behaviour: "The EXIT option is the
default. Following the return or exit, the appropriate action is taken in the caller"
(instrc.xml:2877-2893). The program ends before a delayed CALL ON handler's clause boundary, and a
top level has no caller to raise in.

## Findings

### Important

**R2-1. `to_text`'s and `text_len_inner`'s reference arms now send `DEFAULTNAME` and drop any
condition it raises. Callers that can carry a condition reach them. This reopens N2 through a
reference and regresses outputs where base matched the oracle.**

The change is `reference_text` (`value.rs:518-528`), called from `value.rs:308` and `:776`. The
instrumented mutant `m` prints `DROPPED` there, and running reaches it from these sites
(`mutants/drop-site-*.txt`; the probes are a reference `q = >b` to an object whose `DEFAULTNAME`
raises, under SIGNAL ON SYNTAX):

| site | probe | head | oracle |
|---|---|---|---|
| `numeric_operand` (`run/settings.rs:454`, the `>K>` text rendered untraced) | `numeric digits (q)`, `numeric fuzz (q)` | two sends, then 98.900 | one send, 98.900 (= base) |
| `signal_to_value` | `signal value q` | two sends | one (= base) |
| `builtin::numeric::written_whole` (`numeric.rs:429`) | `5~max(q)`, `5~min(q)` | three sends | one |
| `dispatch::stream::optional_position` | `.stream~new('zz')~charin(q)` | two sends | one |
| `render` from `apply_trigger` (PARSE) | `parse var x (q) y` | sends, drops, prints `returned` | 98.900 |
| RAISE's RC and DESCRIPTION (`run/condition.rs`, `Some(self.to_text(code))`, `self.to_text(value)`) | `n2c/u.rex` | see below | |

The swallowed condition changes the outcome. `c2/once*.rex` use a `DEFAULTNAME` that raises on its
first send only:

    numeric digits (q)     oracle, base: (dn 1) / SYNTAX 98.900 boom.
                           head: (dn 1) (dn 2) (dn 3) / SYNTAX 26.5 DIGITS value must be a positive whole number; found "ONCE".
    signal value q         oracle, base: (dn 1) / SYNTAX 98.900 boom.
                           head: (dn 1) (dn 2) / SYNTAX 16.1 Label "ONCE" not found.

`n2c/u.rex` is the N2 shape: an untraced `raise user zz description (v) return` and
`raise error (v) return`, where `v` holds a reference. Oracle and base print only the handler lines.
Head sends `(noisy dn)` before each one, and for the raising value `(bad dn)`, with the condition
dropped twice (`DROPPED 2` under `m`).

Rexxref's rule gives the oracle's outcome here. A required string value's conversion that raises
raises in the clause (I1's rule), and the untraced RAISE renders nothing (item 2). Concern 2's "no
probe reaches the drop" is false.

The `to_text` arm is not what any witness needs. Mutant `n` (the arms unsent, as at base, with
`object_string_value_text`'s new `VarRef` arm kept) leaves all three new corpus programs, `i1d/k`, `i1f/n`, `i1g/p`
and `i1c/l` o~h equal, and `i1b/g` and `a1` as head prints them (`cmp`). It also matches the oracle
on `c2/once.rex`, `oncesig.rex`, `n2c/u.rex` and `a3/p23`/`p24`. `onceparse.rex` (PARSE) still
answers `returned` there, as base did: the pattern's rendering does not take a conversion that can
raise. That part predates the round.

Fix: keep the two arms unsent, as base had them. A rendering that must send goes through the
propagating path the round added in `object_string_value_text`. Witness `c2/once.rex`,
`c2/oncesig.rex` and `n2c/u.rex`.

## Queue candidates (pre-existing at `d6720f350`, outside the round)

1. **`>=>` reuses the `>>>` rendering** (concern 1). Under TRACE I the oracle sends a third time, and
   a third send that raises is trapped (`c1/d.rex`). Base and head print the line and run on. This
   affects only traced runs.
2. **A RAISE DESCRIPTION object is stored as its unsent rendering.** In `n2b/q9.rex`
   (`raise syntax 93.900 description (o)`, noisy `DEFAULTNAME`), `condition('d')` is `a NOISY` at
   base and head, and the oracle sends and answers `NOISY`. Rexxref: "the language processor
   evaluates the expression to obtain its character string value. This is the description"
   (instrc.xml:2866-2874). Both the rule and the oracle answer `NOISY`. They differ only in when the
   send happens: at RAISE by the rule, at `condition('d')` on the oracle.
3. **`Number~max`/`min` with an object argument.** In `ld/l20.rex` (`numeric digits 5;
   say 5~max(k)`, `k`'s `makeString` `1234567890`), the oracle raises `LOSTDIGITS 1234567890`.
   Base and head send `DEFAULTNAME` and raise `93.904 ... must be a number; found "DNK"`.

## Cleanup

The target dirs `/tmp/claude-1000/t87c-rr2-target-3344571cc`, `-d6720f350`, `-mut`, `-mut2` and
`-dep`, and the archive trees `/tmp/claude-1000/t87c-rr2-src-*`, are deleted after this review.
The probes, outputs, mutant logs and callgrind table are in
`docs/superpowers/records/2026-10-18-phase-9d/t8.7c/review-probes-rr2/`. The raw callgrind files are
in `/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.7c-rr2/`.
