# Task 2 report — the constructor carries state, and the witness proves it

BASE: `a882d3635` (`Fill Task 0's gate statuses, and record Task 1: the bytes live in Body::Instance`).
`$S` = `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/ae5f3c99-11ad-4b29-9dfe-f3d991a1a398/scratchpad/task2`
(small files); `$B` = `/home/moritz/dev/repos/claude-build-scratch/5c-followup-task2` (the BASE
extract and its target directory). Every claim below is **measured** (with the command) or
**inferred**.

## 1. What landed

The tracked files changed are named below, plus `corpus/method-bodies.txt` (section 4); added are
the two corpus programs, their two `sourceline_oracle` companions and this report. By file:

* `rexx-core/src/body.rs` -- `Body::Instance` gains `native: Option<Box<BufferState>>`;
  `pub struct BufferState { bytes: Vec<u8>, capacity: usize, default_size: usize }` with
  `ensure_capacity` (`max(needed, 2 * capacity)`, `MutableBufferClass.cpp:236`-`:248`) and
  `set_buffer_size` (`:679`-`:695`, the three behaviours). `Body::trace` is untouched: its
  `Instance` arm already matches with `..`, and the state holds no `ObjRef`. Exported from
  `rexx-core/src/lib.rs`. The `size_of::<Body>() <= 80` assertion compiled (section 3, C5).
* Every `Body::Instance { .. }` construction gains `native: None` -- `dispatch.rs`'s
  `new_instance`, `run.rs`'s `pool_owner`, and the test constructions in `eval.rs`, `value.rs`,
  and rexx-core's `scope_pools.rs`, `trace.rs`, `uninit.rs`. **The compiler enumerated them**:
  `cargo build --release -p rexx-exec --bin rexx-run`, `cargo test --release -p rexx-core` and
  `cargo test --release -p rexx-exec --lib` are each rc 0; the build's stderr has no `error` and
  no `warning` line (`$S/build1.err`; the test runs' own stderr was not read for warnings --
  clippy in the fast checks is the instrument for those; `$S/test-core.out`, `$S/test-lib.out`).
* `rexx-exec/src/value.rs` -- `Interp::buffer(&self, ObjRef) -> Option<&BufferState>` and
  `buffer_mut` beside `array_slots`/`array_body`; `redirect_of` gets `Body::Instance { native:
  Some(_), .. } => Redirect::None` **ahead of** the `name: None` arm; `to_text`, `try_text` and
  `text_len_inner` each get an arm answering `state.bytes` ahead of the `name` arm.
  `text_len_agrees_with_to_text` now builds a buffer in both name shapes, and
  `try_text_answers_only_where_the_bytes_already_exist` asserts a named and an unnamed buffer
  answer `abcdef` through `try_text`, `to_text` and `text_len`.
* `rexx-exec/src/dispatch.rs` -- `native_mutable_buffer_new` keeps the string and derives
  `default_size` (second argument or `BUFFER_DEFAULT_LENGTH` = 256, `MutableBufferClass.hpp:150`)
  and `capacity = max(default_size, initial.len())`, reserving with `try_reserve_exact`; seven
  `NATIVE_METHODS` rows `APPEND` (`Counted`), `DELSTR` (2), `ENDSWITH` (1), `GETBUFFERSIZE` (0),
  `LENGTH` (0), `SETBUFFERSIZE` (1), `STRING` (0) -- **placed in the table's own alphabetical
  position between `Method` and `MutexSemaphore`, not beside `("String", "LENGTH", …)` as the
  brief said**, because the table is sorted by class and `String` is not where `MutableBuffer`
  sorts; the seven bodies, plus `buffer_state`/`buffer_state_mut` (the `None` refusal is
  `Loud::native_method`); `optional_length_argument` now answers `Option<usize>` (`native_capacity_init`
  still discards the value); `required_length_argument` (missing is
  93.903) and `optional_position_argument` (93.924) added beside it.
* `rexx-exec/src/builtin/string.rs` -- `pub(crate) fn delete_range(&mut Vec<u8>, begin, Option<usize>)`
  extracted; the builtin `delstr` is now its argument prologue plus one call to it, and the method
  calls the same function. No other core moved.
* `corpus/lang/mutablebuffer_state.rex`, `corpus/lang/mutablebuffer_instance.rex`; two rows in
  `corpus/unfiled.txt` (`Task 4 files it into phase-5c.txt`); and
  `rexx-parse/tests/sourceline_oracle/mutablebuffer_{state,instance}.txt`, generated with the
  driver in `sourceline_oracle.rs`'s module comment run on the scratch copies (`$S/srclines/`;
  driver rc 0 both, `count 53` and `count 24`, bodies `diff`-identical to the programs) --
  `sourceline_matches_the_interpreter_for_every_corpus_program` panics without them. The brief did
  not name this file pair; the test's own message does.

## 2. The witnesses and the oracle

### 2.1 `corpus/lang/mutablebuffer_state.rex`

The brief's nine lines, then the truncation case, the two `setBufferSize(0)` shrink cases, the
growth sequence, and multi-argument `append` / two-argument and omitted-argument `delstr` /
`endsWith` negatives / the receiver answered by the mutators. `buf`, never `b`. No `say buf`, no
concatenation of a buffer, no `makeString`.

```rexx
/* A MutableBuffer keeps the contents and the capacity it is built with:
   length, string, endsWith, append, delstr, getBufferSize and setBufferSize
   each answer what the oracle answers. Rendering the buffer itself -- say,
   concatenation, makeString -- is deliberately absent. */
buf = .MutableBuffer~new('abcdef')
say buf~length
say buf~string
say buf~endsWith('ef')
buf~append('ghi')
say buf~length
say buf~string
buf~delstr(9)
say buf~string
say .MutableBuffer~new('x', 999)~getBufferSize
say .MutableBuffer~new(copies('x', 400))~getBufferSize
buf~setBufferSize(500)
say buf~length buf~getBufferSize
/* setBufferSize below the contents truncates them and the capacity follows. */
cut = .MutableBuffer~new('abcdef', 100)
cut~setBufferSize(3)
say cut~length cut~getBufferSize cut~string
/* setBufferSize(0) empties the buffer and takes the capacity back to the
   constructor's default, whether that default was given or implied. */
big = .MutableBuffer~new(copies('x', 400))
big~setBufferSize(0)
say big~length big~getBufferSize
grown = .MutableBuffer~new('abc', 500)
grown~append(copies('y', 600))
grown~append(copies('y', 600))
say grown~length grown~getBufferSize
grown~setBufferSize(0)
say grown~length grown~getBufferSize
/* An append that outgrows the capacity doubles it, or takes what it needs. */
step = .MutableBuffer~new('', 10)
sizes = ''
do 8
  step~append('1234567')
  sizes = sizes step~getBufferSize
end
say strip(sizes)
/* append takes several arguments; a delstr start past the end deletes
   nothing and an omitted start is 1; endsWith is 0 for a suffix that is
   absent or longer than the contents; the mutators answer the receiver. */
buf~append('i', 'jk')
say buf~string buf~length
buf~delstr(2, 3)
say buf~string
buf~delstr(99)
say buf~string
say buf~endsWith('k') buf~endsWith('x') buf~endsWith(copies('z', 50))
say buf~setBufferSize(4)~string
say buf~delstr(, 2)~string
say buf~append('h')~length
```

**Measured on the oracle** (`$S/run_oracle.sh $S/oracle/w1`, the wrapper from the brief, from that
fresh directory): **rc 0, stderr empty**, stdout:

```text
6
abcdef
1
9
abcdefghi
abcdefgh
999
400
8 500
3 3 abc
0 256
1203 2000
0 500
10 20 40 40 40 80 80 80
abcdefghijk 11
aefghijk
aefghijk
1 0 0
aefg
fg
3
```

**Measured on this build, both engines** (`$S/run_crate.sh target/release/rexx-run new
$S/probes/new-w1`, `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`): `cmp` of `.out`, `.err` and
`.rc` against the oracle's three files -- **identical on all three descriptors on both engines**
(`OK` lines in the run log, `$S/probes/new-w1.log`).

### 2.2 `corpus/lang/mutablebuffer_instance.rex`

The two silent divergences Task 1 found, plus the named-buffer rendering rule from
`task-1-report.md` §4.2 fact 3, in one program: `trace i` around `x = buf`; `~objectName=` then
`~string` and `~objectName`; a `::class sub subclass MutableBuffer` whose `init` does `expose n;
n = 42`, read back through a `::method n`.

```rexx
/* A MutableBuffer is an ordinary instance around its bytes: a trace value
   line shows the contents, a name set through objectName= changes what
   objectName answers and not what string answers, and a subclass keeps an
   object variable pool of its own. Rendering the buffer through say is
   deliberately absent. */
buf = .MutableBuffer~new('abcdef')
trace i
x = buf
trace o
say x~length
buf~objectName = 'named'
say buf~string
say buf~objectName
sub = .sub~new('pq')
say sub~n sub~length
sub~append('r')
say sub~n sub~length sub~string
::class sub subclass MutableBuffer
::method init
  expose n
  n = 42
::method n
  expose n
  return n
```

**One wrong draft on the way, caught by the oracle.** The first version had no `::method n` and
read `sub~n` directly, which is Error 97.1 `Object "pq" does not understand message "N"` at rc 159
-- an object variable is not a method. Task 1's `sub.rex` must have carried a getter; this one
does.

**Measured on the oracle** (`$S/run_oracle.sh $S/oracle/w2`): **rc 0**, stdout

```text
6
abcdef
named
42 2
42 3 pqr
```

and stderr the trace lines (the oracle writes trace to stderr):

```text
     8 *-* x = buf
       >V>   BUF => "abcdef"
       >>>   "abcdef"
       >=>   X <= "abcdef"
     9 *-* trace o
```

**Measured on this build, both engines**: identical on all three descriptors
(`$S/probes/new-w2.log`). The `>V>   BUF => "abcdef"` line is the rendering fix; against BASE the
same line reads `"a MutableBuffer"` (section 3, C2).

### 2.3 Refusals measured before they were written

Every refusal below was run on the oracle first (`$S/oracle/r/r*.rex`, one two-line program each,
`buf = .MutableBuffer~new('abc')` then the probe line; `$S/run_oracle.sh $S/oracle/r`), then on
this build under both engines (`$S/probes/new-r/`). Verdict per probe: `cmp` on stdout and rc,
and stderr compared after normalising the absolute path the traceback embeds (the two sides ran
from different directories). **Every `r*` probe agrees on all three descriptors modulo that
path**, both engines: the normalised stderr diff over all of them is empty. Of the seven `mb_*`
probes (section 4's zero-argument sends) the one that differs beyond the path is `mb_delStr`, and
that difference is the predicted one.

| probe | oracle | which raiser here |
|---|---|---|
| `buf~length(1)`, `~string(1)`, `~getBufferSize(1)` | 93.902 `0 expected`, rc 163 | `Arity::Fixed(0)`, the table |
| `buf~endsWith('a','b')` | 93.902 `1 expected` | `Arity::Fixed(1)` |
| `buf~delstr(1,1,1)` | 93.902 `2 expected` | `Arity::Fixed(2)` |
| `buf~setBufferSize(1,2)` | 93.902 `1 expected` | `Arity::Fixed(1)` |
| `buf~endsWith`, `buf~endsWith(,)` | 88.901 `Missing argument; argument match is required.`, rc 168 | `Raised::missing_named_argument("match")` |
| `buf~endsWith(.nil)` | 88.909 `Argument match must have a string value.`, rc 168 | `required_string_named_argument(.., "match")` |
| `buf~append` | 93.903 `argument 1 is required`, rc 163 | `Raised::missing_method_argument(1)` |
| `buf~append(, 'x')` | 93.903 argument 1 | the same, per omitted position |
| `buf~append(.nil)` / `buf~append('a', .nil)` | 88.909 `Argument 1` / `Argument 2 must have a string value.`, rc 168 | `required_string_argument(.., index + 1)` |
| `buf~delstr('x')`, `(0)`, `(.nil)`, `('1.5')` | 93.924 `Invalid position argument specified; found "x"` / `"0"` / `"The NIL object"` / `"1.5"`, rc 163 | `optional_position_argument` |
| `buf~delstr(1,'y')`, `(1,-1)`, `(1,.nil)`, `(2, 99999999999999999999)` | 93.923 `Invalid length argument specified; found …`, rc 163 | `optional_length_argument` |
| `buf~setBufferSize` | 93.903 argument 1, rc 163 | `required_length_argument` |
| `buf~setBufferSize('x')`, `(-1)`, `(99999999999999999999)` | 93.923, rc 163 | the same |
| `.MutableBuffer~new('a', -1)`, `('a', 'x')` | 93.923, rc 163 | unchanged constructor checks |
| `.MutableBuffer~new(.nil)` | 88.909 `Argument 1`, rc 168 | unchanged |

Semantics measured the same way and agreeing on both engines: `append(12)` → `abc12 5`;
`endsWith(3)` on `abc` is `0` and on `ab3` is `1`; **`endsWith('')` is `0`** on `abc` and on the
empty buffer (`primitiveMatch`'s `len == 0` arm, `:1615`); `setBufferSize(' 2 ')` and `('2.0')`
both truncate to `ab 2`; `setBufferSize(256)` on a 256-capacity buffer leaves `abc 256`;
`delstr(1,0)` leaves `abc`; `delstr(,)` empties; `delstr(,1)` → `bc`; `append('x',)` → `abcx`
(the trailing omission is dropped from the count on both sides).

## 3. Controls, each predicted before it ran

Predictions for C1-C2 are in `$S/probes/base-prediction.txt`, for M1-M4 in
`$S/mutations/prediction.txt`, both written before the runs they name. C3-C6's predictions were
not written to a file of their own; each is the brief's or Task 1's stated expectation (C3: Task 1
§4.2 fact 1; C4: the brief's "red before your edit to it, green after"; C5: Task 1 §2.5's B-prime
row; C6: the brief's "the builtin's existing tests are the control"). BASE binary: `$B/target-base/
release/rexx-run` (section 6). Witness runs are both engines, three descriptors, `cmp` against the
oracle files.

| # | control | predicted | read | verdict |
|---|---|---|---|---|
| C1 | `mutablebuffer_state.rex` on BASE | rc 120, stdout empty, stderr exactly `rexx-exec: method "LENGTH" of class "MutableBuffer" is not implemented (Phase 5)`, both engines | exactly that on `ir` and `tree-walker` (`$S/probes/base-w1/`) | **confirmed** -- the program tests the change |
| C2 | `mutablebuffer_instance.rex` on BASE | rc 120, stdout empty, stderr the trace lines with `"a MutableBuffer"` then the `LENGTH` loud line | `>V>   BUF => "a MutableBuffer"`, `>>>   "a MutableBuffer"`, `>=>   X <= "a MutableBuffer"`, then the loud line; rc 120; both engines (`$S/probes/base-w2/`) | **confirmed** -- Task 1's silent divergence, reproduced on BASE |
| C3 | `say buf`, `'x' \|\| buf`, `buf~makeString` on this build | rc 120, stderr `method "MAKESTRING" of class "MutableBuffer"` loud, both engines | all three, both engines (`$S/probes/new-loud/`) | **confirmed** -- the rendering arms do not make the conversion answer |
| C4 | the old text of `a_constructor_taking_arguments_answers_an_instance_and_refuses_its_state` against the new code | red at the `MutableBuffer` row: expected rc 120, got `3` | `assertion left == right failed: .MutableBuffer~new('abc')`, `left: (0, "3\n")`, `right: (120, "")`; `1 failed` (`$S/mutations/old-test.out`); the new text then restored byte-identical and green in `--lib` (779 passed) | **confirmed** |
| C5 | `size_of::<Body>()` | assertion at `body.rs` compiles; width 80 (Task 1 §2.5's B-prime) | `cargo build` rc 0; probe line `const _: [(); 0] = [(); size_of::<Body>()];` under `cargo check -p rexx-core`: E0308 `found one with a size of 80`, and only that error plus the `could not compile` summary (`$S/size-probe.err`); probe removed, `body.rs` sha256 unchanged | **confirmed**, 80 |
| C6 | `delstr` extraction: the builtin's own tests | green after the extraction | `cargo test --release -p rexx-exec --lib`: `779 passed; 0 failed` (`$S/test-lib.out`), the `string.rs` `mod tests` `DELSTR` rows among them | **confirmed** (the corpus differential in the fast checks below is the second control) |
| M1 | delete the `*native = Some(..)` write in the constructor, field kept | both witnesses rc 120, stdout empty, stderr the `LENGTH` loud line (w2 with the `"a MutableBuffer"` trace lines first) -- indistinguishable from BASE; `--lib` exactly one failure, the constructor test; `method_bodies` fails | exactly that: `RC(oracle 0 vs 120)` on all four witness runs, stderr as predicted; `--lib` `778 passed; 1 failed`, the one being `a_constructor_taking_arguments…` (`left: (120, "MutableBuffer\n", "…LENGTH…")`); `method_bodies` `15 passed; 1 failed`, `no_row_started_diverging_or_stopped_answering` | **confirmed** |
| M2 | keep the write, store `bytes: Vec::new()` (state present, empty) | w1 rc 0, stderr empty, stdout diverging from line 1 (`0` against `6`); w2 line 1 `0`, trace lines `""`; `--lib` the same one failure (`0` against `3`); `method_bodies` fails | w1 `STDOUT-DIFF` only (rc 0, stderr 0 bytes), `diff` opens `1,6c1,6` with `0` against `6`; w2 stdout `0 / (empty) / named / 42 0 / 42 1 r`, trace `>V>   BUF => ""`; `--lib` `778 passed; 1 failed` with `left: (0, "MutableBuffer\n0\n", "")`; `method_bodies` `15 passed; 1 failed`, the same test | **confirmed** -- a stub that reads its arguments still cannot pass the witness |
| M3 | `ensure_capacity` grows to `needed` only (no doubling) | w1 rc 0, exactly two stdout lines differ: `1203 2000` → `1203 1203`, `10 20 40 40 40 80 80 80` → `10 14 21 28 35 42 49 56`; w2 identical; `--lib` all pass; `method_bodies` passes | `diff`: `12c12` and `14c14`, exactly those two lines, both engines identical to each other; w2 `OK` both engines; `--lib` `779 passed; 0 failed`; `method_bodies` `16 passed; 0 failed` | **confirmed** -- **only the witness catches M3**; this is the "adds coverage" reading the brief asks for, over the method-body table and the unit tests together |
| M4 | delete `redirect_of`'s `native: Some(_) => Redirect::None` arm | w1 identical; w2 stdout identical, stderr's three value lines `"a MutableBuffer"`; `--lib` exactly one failure, `try_text_answers_only_where_the_bytes_already_exist` on its `to_text` assertion; `method_bodies` passes | w1 `OK` both engines; w2 `OK STDERR-DIFF`, the three lines reading `"a MutableBuffer"`, stdout identical; `--lib` `778 passed; 1 failed`, that test, `left: [97, 110, 32, 79, 98, 106, 101, 99, 116]` (`an Object`) against `abcdef`; `method_bodies` `16 passed` | **confirmed** -- the trace-line divergence is what the table cannot see and the instance witness does |

After M4 the sources were restored from `$S/backup/` by `cp` and `touch` (each restore verified
`sha256sum` against `$S/backup/sha256.txt`: "byte-identical" all four times) and the release binary
rebuilt: its sha256 is `5f827c81569af565ef3f3eed83bf3a7877af85a74b2b15c4921b161bb01dd70c`, the same
as before the first mutation (`$S/rexx-run.task2.sha256`), so the binary the fast checks and the
witness readings above stand on is the committed one. Each mutation's own build, witness outputs,
`--lib` and `method_bodies` logs are under `$S/mutations/M<n>/`.

**What the suite without the new assertions catches, per mutation** (the "can fail" against "adds
coverage" reading): M1 and M2 are caught by the `method-bodies.txt` drift gate on its own
(`answers` → `loud` / `diverge` is a regression), so the corrected unit test and the witness add a
second and third catcher there; M3 is caught by **nothing but `mutablebuffer_state.rex`**; M4 is
caught by the `try_text` unit test and by `mutablebuffer_instance.rex`, and by no table. The two
witnesses are not yet run by any gate -- they are in `corpus/unfiled.txt` until Task 4 files them --
so until then M3 has no automated catcher at all; that is the plan's own sequencing, recorded here so
Task 4 knows what its filing turns on.

## 4. `corpus/method-bodies.txt` row moves

**Predicted, written before the refresh ran** (receiver `.MutableBuffer~new('abc')`, a
zero-argument send under `say`; the oracle's answers for each are in 2.3's `mb_*` rows):

| row | before | predicted after | why |
|---|---|---|---|
| `length` | `loud` `LENGTH` | `answers` `rc 0` | both print `3` |
| `string` | `loud` `STRING` | `answers` `rc 0` | both print `abc` |
| `getBufferSize` | `loud` `GETBUFFERSIZE` | `answers` `rc 0` | both print `256` |
| `endsWith` | `loud` `ENDSWITH` | `answers` `rc 168` | both raise 88.901 naming `match` |
| `append` | `loud` `APPEND` | `answers` `rc 163` | both raise 93.903 argument 1 |
| `setBufferSize` | `loud` `SETBUFFERSIZE` | `answers` `rc 163` | both raise 93.903 argument 1 |
| `delStr` | `loud` `DELSTR` | **stays `loud`**, evidence `method "MAKESTRING" of class "MutableBuffer"` | the oracle prints an empty line (the emptied buffer, rendered); here the send answers the receiver and `say` then reaches the unbound `MAKESTRING` |

Every other row -- `MutableBuffer`'s remaining instance rows, its class-arm `new`, and every other
class -- unchanged. No row to `diverge`.

**Measured.** `REXX_METHOD_BODIES_REFRESH=1 cargo test --release -p rexx-exec --test method_bodies`
from `rust/`: rc 0, `test result: ok. 16 passed; 0 failed`, 83.08 s (`$S/refresh.{out,err,rc}`).
`git diff --stat -- corpus/method-bodies.txt`: `7 insertions(+), 7 deletions(-)`, and every changed
line is a `MutableBuffer` row:

```text
-MutableBuffer	append	instance	loud	method "APPEND" of class "MutableBuffer"
+MutableBuffer	append	instance	answers	rc 163
-MutableBuffer	delStr	instance	loud	method "DELSTR" of class "MutableBuffer"
+MutableBuffer	delStr	instance	loud	method "MAKESTRING" of class "MutableBuffer"
-MutableBuffer	endsWith	instance	loud	method "ENDSWITH" of class "MutableBuffer"
+MutableBuffer	endsWith	instance	answers	rc 168
-MutableBuffer	getBufferSize	instance	loud	method "GETBUFFERSIZE" of class "MutableBuffer"
+MutableBuffer	getBufferSize	instance	answers	rc 0
-MutableBuffer	length	instance	loud	method "LENGTH" of class "MutableBuffer"
+MutableBuffer	length	instance	answers	rc 0
-MutableBuffer	setBufferSize	instance	loud	method "SETBUFFERSIZE" of class "MutableBuffer"
+MutableBuffer	setBufferSize	instance	answers	rc 163
-MutableBuffer	string	instance	loud	method "STRING" of class "MutableBuffer"
+MutableBuffer	string	instance	answers	rc 0
```

Row for row the prediction, `rc` values included; the `delStr` row moved exactly as predicted --
still `loud`, now naming `MAKESTRING`. `grep -c` of `diverge|unstable` over the `+` lines: 0. **No
row moved to `diverge`; no row of any other class moved.** The `mb_*` probes in `$S/probes/new-mb/`
are the same seven sends run by hand on both engines and agree with this reading (the `delStr` one
is the single probe of section 2.3 whose stderr differs beyond the path: the crate's
`MAKESTRING` loud line where the oracle prints an empty stdout line).

## 5. Corrections to prose and tests

* `native_mutable_buffer_new`'s doc said *"an instance carrying neither"* and *"The buffer's
  contents are not kept, and nothing that would read them answers -- which is what keeps the
  rendering honest …"*, and cited the test as asserting the pair. Both sentences are false now and
  are gone; the doc states what the constructor derives (`default_size`, `capacity`) and keeps the
  `INIT`-argument paragraph, which is still true.
* `a_constructor_taking_arguments_answers_an_instance_and_refuses_its_state`: the
  `MutableBuffer` row left the refusal loop and became `assert_eq!(both_engines("o =
  .MutableBuffer~new('abc')\nsay o~class~id\nsay o~length\n"), (0, "MutableBuffer\n3\n", ""))`.
  Its doc no longer says the crate holds none of the state. The `say .MutableBuffer~new('abc')`
  rc 120 assertion is kept and **sharpened**: it now requires the stderr to start with `rexx-exec:
  method "MAKESTRING" of class "MutableBuffer"`, and its comment gives the current reason (`say`
  reaches the buffer through the required-string protocol's unbound `MAKESTRING`) rather than the
  old one (that an answer would have been `a MutableBuffer`).
* `optional_length_argument`'s doc: *"an omitted argument is the default"* → *"`None` for an
  omitted argument"*, matching its new return type.
* `Body::Instance`'s doc gains the `native` sentence; the value.rs comments *"Reached only with a
  name set"* stay true because the buffer arm now sits ahead of them and takes the buffer first.
* The two `try_text`/`text_len` tests gained the buffer cases described in section 1.

## 6. Files created outside the tree

Nothing deleted anywhere; nothing written under `rust/` or `docs/` except the committed files and
cargo's own `target/`.

`$B` = `/home/moritz/dev/repos/claude-build-scratch/5c-followup-task2/`:
`base/` (`git archive a882d3635`, 73 MB), `target-base/` (its `CARGO_TARGET_DIR`;
`release/rexx-run` sha256 `42430a364140de921ac465ceb2451d96bc973547be6f4b60ab5ca854c0e9384e`,
27059224 bytes), `build-base.{out,err,rc}` (rc 0).

`$S` = `…/scratchpad/task2/`: `run_oracle.sh`, `run_crate.sh`, `rep.py` (the exact-once
replacer every source edit went through) and `e/` (its old/new snippets); `oracle/{w1,w2,r,mb}/`
(the programs and their oracle `.out/.err/.rc`); `probes/base-prediction.txt`,
`probes/base-w1/`, `probes/base-w2/` (BASE binary runs), `probes/new-{w1,w2,r,mb,loud}/` and
`probes/new-*.log` (this build); `srclines/` (the sourceline driver, raw captures, generated
`.txt`); `method-bodies.before.txt`; `rexx-run.task2.sha256`; `build1.err`; `test-core.*`,
`test-lib.*`; `refresh.*`; `size-probe.{out,err}` and `body.rs.sha256.before-probe`;
`backup/` (the three sources and their sha256 list the mutation restores were checked against);
`mutations/` (section 3: `prediction.txt`, `m<n>.{old,new}`, `run_mutations.sh`, `status.txt`,
`pid`, `M<n>/`, `old-test.{out,err}`); `fastchecks/` (`run.sh`, `status.txt`, `pid`, one
`.out/.err` pair per check); `commit-msg.txt`; `gates/` (`run_gates.sh`, `status.txt`, `pid`,
`g<n>.{out,err}`).

## 7. Open questions

* **`DELETE` is `mydelete` under a second name (`Setup.cpp:1427`) and is not bound here.** The brief
  names seven methods and `delete` is not one; binding it is one table row pointing at
  `native_mutable_buffer_delstr`, and Task 3's mutator commit is where the plan puts it. Its
  `method-bodies.txt` row stays `loud` `DELETE` until then.
* **The brief's mutation prediction describes M2, not M1.** "Delete the `native` write (keep the
  field): the witness must fail at line 1 (`length` 0 against 6)" -- under this representation an
  instance whose `native` is `None` takes the refusal path, so deleting the write reproduces BASE
  (rc 120) rather than a wrong `0`. The wrong-`0` shape is a state that exists and is empty, which
  is M2. Both were run and both are in section 3.
* **The rows sit in `NATIVE_METHODS`'s alphabetical position**, not beside `("String", "LENGTH",
  …)` as the brief said; the table is ordered by class and a reviewer reading it top to bottom
  would look for `MutableBuffer` between `Method` and `MutexSemaphore`.
* **The `Vec`'s reservation follows the observable capacity** through `try_reserve_exact` (growth,
  fallible, 5.0 on failure as the builtins do) and `shrink_to` (shrinking, infallible). A shrink
  the allocator refuses aborts rather than raising; no probe reaches that and this crate has no
  raise for it.
* **`mydelete`'s default range is `capacity - begin`, not `length - begin`** (`data->getDataLength()`
  is the `BufferClass`'s size, `MutableBufferClass.cpp:650`). Unobservable: the range is only read
  when `begin < dataLength <= capacity`, where both mean "to the end", and `delete_range` takes
  `None` for that. Recorded so Task 3's `delete` does not re-derive it.
* **`endsWith('')` is `0` on the oracle** and is implemented so. Whether `String~endsWith('')` is
  also `0` was not measured and is 5c's `String` surface, not this task's.
* **The `sourceline_oracle` companion files are a per-program cost the plan does not mention.**
  Every new `corpus/lang` program needs one, generated by the driver in that test's module
  comment; Task 3's family witnesses will each need theirs.

## Fast checks before the commit

Run from `rust/` by `$S/fastchecks/run.sh` on the restored tree (section 3's final rebuild), each
status unpiped (`$S/fastchecks/status.txt`), run counts read from each binary's own `test result`
line:

| check | exit | run count |
|---|---|---|
| `cargo fmt --all --check` | 0 | (no output) |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | `Finished` with no warning |
| `cargo test --release -p rexx-exec --test corpus --test method_bodies --test coverage` | 0 | `18 passed; 1 ignored` / `20 passed` / `16 passed` |
| `cargo test --release -p rexx-exec --lib` | 0 | `779 passed` |
| `cargo test --release -p rexx-parse --test sourceline_oracle --test program` | 0 | `28 passed` / `1 passed` (the two binaries; both walk `corpus/lang`, so both see the new programs) |

`Cargo.lock` is unchanged (`git diff --stat -- rust/Cargo.lock` empty) and is not staged.

## Gates

Run from `rust/` by a background job writing each status unpiped to a file as it goes, the commit
sha as its first line, with a pidfile. Statuses at `$S/gates/status.txt`.

| # | command | exit |
|---|---|---|
| 1 | `cargo fmt --all --check` | **0** |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | **0** |
| 3 | `cargo test --release --workspace --no-fail-fast` | **101** |
| 4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` | **101** |
| 5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | **101** |
| 6 | `REXX_PHASE_GATE=5c REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast` | **0** |
| 7 | `REXX_PHASE_GATE=5d REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast` | **0** |

**Read by the controller from `$S/gates/status.txt`** (first line `2d533034a…`, last line
`finished`). **G3, G4 and G5 are 101 on one test**, the same in all three:
`refusal_sites.rs`'s `the_table_holds_every_constructor_the_source_defines` — `corpus/refusal-sites.tsv`
cites every `Raised`/`Loud` constructor's definition by `file:line`, and this task's one-line
`native: None` at `run.rs:3175` moved 19 `run.rs` definitions down by one. Parsed from the panic:
19 rows in the source not in the table, 19 in the table not in the source, the same 19 `(kind,
name)` pairs, every delta `+1`, no constructor added or removed, no surface changed. The fast
checks did not run that binary. Re-derived by the controller in the follow-up commit; nothing else
in G3–G5 failed.
