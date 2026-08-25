# Task 14 -- required string values

**Status: DONE_WITH_CONCERNS**, after three review rounds. The protocol lands, every context the
section names and this phase can reach goes through it, the corpus is **171 of 171** against the oracle
with `ir_dual` holding the two engines to each other over the same set, gate table C's `reqstr` row
reads `agree`, both controls are run and recorded, and the slowest own-contribution ratio across all
three sittings is 1.007835 (round 0's `compound`/ir; the two fix-round sittings read 1.002034 and
1.000005).

Round 1 and round 2 changed code and each has its own `# Fix round` section below. **Round 3 changed
no behaviour**: it exists because round 2's answer to its own most severe finding was in a commit
message and not in this file, and its work is recorded inside the `# Fix round 2` section, which is
what it documents.

**The concerns as they now stand**, since the fix rounds have closed and replaced some of them: the
contexts whose instruction another phase owns; the places where a refusal I left standing is what
keeps a wrong answer out; the latch check being debug-only, and the one arming site its corpus witness
covers; the re-derivation's own blind spot; and `dispatchclass`'s resolution floor, which is a fact
about the guard rather than about this task. **The concern section at the end of each round is that
round's list, and the last one is live**; nothing earlier in the file was rewritten to match a later
round, so a paragraph in an earlier round that a later round corrects says so where it stands.

Base: `df799fee0`.

| commit | what |
|---|---|
| `b305019ec` | the protocol, the latch, and every context that becomes a dispatch site |
| `93099cf99` | `~string`, `~request`, `~objectName`, `~objectName=`, `String~makeString` |
| `b15552ec3` | nine corpus programs, the in-crate refusal tests, the `object_operand_tests` re-derivation |
| `6f7899514` | set the counter before the trap that reports it -- found by running control 1 |
| `127fb74b8` | the sitting, and two comments that enumerated a repo set |

`.superpowers/` is git-ignored in this checkout (`.gitignore:30`), as it was for the thirteen tasks
before this one, so this report is on disk at the path the brief named and is in no commit.

---

## What I implemented

`provide.xml` `reqstr` specifies one rule: `request("STRING")`, then the receiver's `makeString`, then
the NOSTRING condition if it is trapped, else `defaultName`. The crate had the last limb only, by
rendering. `Interp::required_string_value` is the whole protocol, and every context the section lists
that this phase can reach now calls it.

The C++ this is derived from is `RexxInternalObject::requestString` (`classes/ObjectClass.cpp:1235`),
its `requiredString()` sibling at `:1341` for method arguments, and `RexxObject::requestRexx` at
`:1912` for the Rexx-level `~request`. `requestStringNoNOSTRING` at `:1302`, which the spec's own row
cites, has **no caller anywhere in the C++ tree** -- `grep -rn "requestStringNoNOSTRING()"
--include=*.cpp --include=*.hpp .` from `interpreter/` matches its declaration and its definition and
nothing else -- so it is not modelled.

### The contexts

The section's own list, and where each one calls the protocol:

| context | site |
|---|---|
| `SAY` | `Interp::say_evaluated` (`run.rs`), both engines |
| `PUSH`/`QUEUE` | `Interp::queue_evaluated` (`run.rs`), both engines |
| `INTERPRET` | `step`'s `Interpret` arm (`run.rs`) |
| `DO`'s `exprr` and `exprf` | `Interp::accept_header_value`'s `Count` arm for `exprr`, and its `For`/`OverFor` arms for `exprf` (`run.rs`) |
| substituted compound-variable tails | `Interp::append_tails` (`stem.rs`) |
| `ADDRESS`'s environment name | `Interp::exec_address` (`run.rs`) |
| `ARG`/`PARSE`/`PULL` | `Interp::parse_strings` and `Interp::argument_text` (`parse_template.rs`) |
| parenthesised `CALL` targets | `step`'s `Call::Dynamic` arm (`run.rs`) |
| `DROP`/`EXPOSE`/`PROCEDURE` lists | the three `VariableRef::Indirect` arms (`run.rs`) |
| `NUMERIC`'s three values | `Interp::numeric_operand` and the `FormValue` arm (`run.rs`) |
| `SIGNAL VALUE` | `Interp::signal_to_value` (`run.rs`) |
| `TRACE VALUE` | `exec_trace`'s `Trace::Value` arm (`run.rs`) |
| builtin arguments | `builtin::run` (`builtin.rs`), once over every position `raw_argument_positions` does not exempt |
| dyadic operators, right operand | `Interp::apply_binary` and `Interp::arith_general_body` (`eval.rs`) |
| method arguments, "all other methods" | `required_string_argument` (`dispatch.rs`) |

Three of the section's contexts are **not** sites, because the instruction itself is refused in this
phase and the refusal names another owner:

* a command to an external environment, and a command on an `ADDRESS` instruction -- Phase 7;
* the `OPTIONS` instruction -- 5c, by the spec's own amended D31.

`dispatch.rs`'s `a_reqstr_context_whose_instruction_is_another_phases_is_still_loud` asserts all three
stay loud with a `makeString` receiver in place, which is the only instrument for them: a refusal the
oracle does not share cannot be a corpus row.

The section's other half of the method rule -- String's arithmetic, comparison and concatenation
methods falling back to `~string` rather than raising -- has **no reachable site**: `NATIVE_METHODS`
gives `String`'s own scope no method that takes an argument -- every row there is
`Arity::Fixed(0)` -- so `'abc'~pos(.k)` reaches the not-implemented gate before its argument is
looked at, and there is no site for the fallback to happen at.

### Which operand an error names, per site

This is the part that took the most measurement, because it is not uniform and each site's own C++
decides it. Every row below is three descriptors against the oracle, from a fresh empty directory,
with a class-side `makeString` on the object:

| site | the message names | measured |
|---|---|---|
| `DO`'s `exprr`/`exprf` | the **object** | `do .K` with `makeString` `'xx'` is 26.2 `found "The K class"` |
| `NUMERIC DIGITS`/`FUZZ`/`FORM VALUE` | the **object** | `numeric digits .K` with `'xx'` is 26.5 `found "The K class"` |
| arithmetic operators | the **object** | `'2' + .K` with `'xx'` is 41.1 `Nonnumeric value ("The K class")` |
| builtin numeric arguments | the **object** | `substr(.A, .B)` with `.B` `'x'` is 40.12 `found "The B class"` |
| `SIGNAL VALUE` | the **conversion** | `signal value .K` with `'NOSUCH'` is 16.1 `Label "NOSUCH" not found.` |
| `TRACE VALUE` | the **conversion** | `trace value .K` with `'ZZ'` is 24.1 `found "Z"` |
| logical operators | the **conversion** | `1 & .K` with `'3'` is 34.901 `found "3"` |
| builtin pad arguments | the **conversion** | `padArgument` converts and quotes its answer |

`RexxInstructionNumeric::execute` reports `result` at each of `:105`, `:141` and `:189`;
`DoBlockComponents.cpp` reports `result` at `:99` and `:106`; `StringClass::arith` is handed
`otherObj`. The two that name the conversion do so because the value they check *is* the converted
string.

### And which value a trace line shows

`>K>` keyword lines name the **object**, `>>>` value lines name the **conversion**. Measured, `trace r`
on both sides:

* `numeric digits .K` with `makeString` `12` traces `>K>   "DIGITS" => "The K class"` and then answers
  `12` to `DIGITS()`;
* `parse value .K with a b` with `'p q'` traces `>K>   "VALUE" => "The K class"` and then
  `>>>   "p q"`;
* `say .K` with `2` traces `>>>   "2"`; `push .K` with `'pv'` traces `>>>   "pv"`; `address (.K)` with
  `'CMD'` traces `>>>   "CMD"`; `call (.K)` with `'MS'` traces `>>>   "MS"`; `interpret .K` with
  `'nop'` traces `>>>   "nop"`.

So the conversion goes *after* the keyword trace and *before* the value trace at every site, and that
is where each call sits.

### The Rexx-level face

`~string`, `~request`, `~objectName` and `~objectName=` were rc 120 refusals naming `Object`. The
generated class dictionaries already declared all four, so the names resolved and met the
not-implemented gate; each needed a `NATIVE_METHODS` row. `String~makeString` joins them, because
that is what `~request("STRING")` finds on a string receiver.

`~request` follows `requestRexx`: the `MAKE` method first, then the class-id match, then `.nil`. **A
`MAKE` method the dictionaries declare and this crate has no code for stays a refusal.** That matters
because the dictionaries are accurate about *presence*: measured, `.environment~hasMethod("MAKEARRAY")`
is `1` and `.K~hasMethod("MAKEARRAY")` is `0`, and the oracle really does convert the first
(`.environment~request("ARRAY")` is an array at rc 0). So `.environment~request("ARRAY")` is loud and
`.K~request("ARRAY")` is the `.nil` the oracle answers.

`~objectName=` needs storage. A class object gets an override in `ClassRegistry` beside its
`default_names`, which `~defaultName` keeps answering; `.local`, `.environment` and a package object
already carry a stored name in their own `NativeObject`. A receiver with neither -- a string, a number
-- stays loud, because the oracle stores a name there and remembers it, and answering rc 0 while
forgetting it is a wrong answer.

## The design decision on finding 2

**`Interp::to_text` stays total and infallible. The protocol is a separate, fallible pre-step that
answers a *value*.**

```rust
pub(crate) fn required_string_value(&mut self, value: ObjRef) -> Result<ObjRef, Failure>
```

Each context calls it and then renders the answer with the same `to_text` it always did. What I
rejected:

* **Making `to_text` fallible.** `git grep -c 'to_text(' df799fee0 -- 'rust/crates/*'` sums to **149**
  across 18 files, and most are not `reqstr` contexts -- trace echoes, error-message substitutions,
  internal rendering. Each of those would have to invent an
  error path for a conversion it never performs. Worse, `to_text`'s soundness argument is that its
  single scratch buffer is safe *because* `&mut self` prevents a second call while the borrow is live
  (`value.rs`, `Decoded::Text` arm), and dispatching a Rexx method in the middle of that borrow is
  exactly the thing that argument rules out.
* **A second total function that dispatches internally and returns `Cow`.** Same borrow problem:
  `&mut self` held across a Rexx activation.
* **Converting where values enter a context, once, ahead of time.** The protocol has observable side
  effects -- a `makeString` can `say` -- and they have to happen at that point in the program's output
  order, which a hoisted conversion moves. Measured: `say 'before'; say .K` with a `makeString` that
  says `inside makeString` prints `before`, `inside makeString`, `MS`, in that order.

Returning an `ObjRef` rather than bytes is what keeps the rendering code untouched and adds no
allocation to the ordinary path.

### The latch, and why it is checked rather than asserted

`Interp::reqstr_armed` is a monotonic `bool`. It is set when a directive installs a method named
`MAKESTRING` and when a `SIGNAL ON NOSTRING` or `SIGNAL ON ANY` arms a trap. With it clear, the
protocol's first limb cannot answer differently from the value itself and its third cannot fire, so
`required_string_value` returns the value and the walk is skipped.

`signal on any` counts, and that is measured rather than assumed: `signal on any name h` over
`say .environment` runs the handler with `CONDITION('C')` `NOSTRING` at oracle rc 0.

The latch is a correctness claim, so a debug build checks it. `required_string_latch_holds` tests
both of the ways a clear latch can be wrong: it refuses a live NOSTRING-taking trap outright, and it
runs the protocol's conversion limbs and insists the bytes match `to_text`'s. **The bytes, not the
identity** -- the last limb builds a fresh string out of `stringValue()`, so an object never comes back
as itself. The trap half arrived in fix round 1; fix round 1's section records both routes being
proved live by inverting each arming site in turn, which is the only thing that establishes what a
check covers.

The check must not allocate a heap object, because `tests/collect_stress.rs` compares the set of
programs that perform zero collections against a committed list, and a debug-only allocation moved
four programs out of that set. So `StringConversion` carries an array's joined items as `Bytes` rather
than as a built string, and `Interp::required_string_answer` leaves the materialisation to whichever
caller wants an object. That split is why the second version of the check is allocation-free.

## Two structural consequences

**A builtin's arguments are converted once, in position order, before the body runs.** This is not a
convenience: the crate's readers were deliberately reordered (`required_render`'s own doc -- the
numeric and pad arguments first, then the strings), and converting inside the readers would make that
order observable. Measured, oracle rc 0: `substr(.A, .B, .C)` with a saying `makeString` on each
prints `A`, `B`, `C` and then `bcd`, which is position order and not `substr`'s read order. An
argument the builtin does not go on to *use* is converted too, where the oracle fetches it
unconditionally -- `substr('abc', 1, 2, .P)` prints the pad's own line at rc 0 even though the pad is
not needed, because `SUBSTR` reaches its pad through `optional_pad` either way.

**Converting the whole list is *not* right, and fix round 1 corrects that.** A position the oracle
fetches with `stack->peek` is never converted, and `VALUE`'s new value is one; the conversion now runs
over every position `raw_argument_positions` does not exempt. The fix round's own section has the
enumeration and the witness.

That forced `Args`, a pair carrying what the readers use beside what the 40.x messages name, because
the oracle's error path is handed the object and never the conversion. `git diff df799fee0 --
rust/crates | grep -c '^-.*args: &\[Option<ObjRef>\]'` is **79** and the matching
`grep -c "^+.*args: Args<'_>"` is **79**, so it is a one-for-one parameter change across 79
signatures. The two slices are the same slice whenever the latch is clear, so an ordinary call
allocates nothing.

**A compound's `>C>` line moved from the trace echo to the read.** `echo_symbol_read_line` re-derived
the tail key to print the resolved name, which with a converting tail would send a second `makeString`
for one reference. The oracle resolves once: `RexxActivation::evaluateLocalCompoundVariable` builds
`resolved_tail` and traces both lines from it (`execution/RexxActivation.cpp:4793`-`:4802`). So
`read_symbol`'s `Compound` arm emits `>C>` where the key is still in hand. Adjacency survives on the
compiled engine because `assert_read_echoes_follow_their_load` already requires every
`Op::TraceRead` to sit immediately behind its `Op::Load`; that invariant's own doc now says the
adjacency is load-bearing for this reason.

## The re-derivation

`eval.rs`'s `object_operand_tests` and `corpus/lang/message_send_argument_object_not_a_string.rex`
were derived from a coercion audit. Re-derived from the section:

**Every case in `object_operand_tests` is the operand on the *left*, and the section names it
nowhere.** The context is "Rexx dyadic operators when the receiving object (the object to the left of
the operator) is a string", so the operand the protocol converts is the one on the **right**: the
receiver's own method asks its argument for a string value. An object on the left is not converted at
all, it is sent a message. So none of those cases is a `reqstr` row, and all of them stay. The
asymmetry is measured: `.array + 1` is 97.1 where `1 + .array` is 41.1, and
`say (.array == 'The Array class')` is `0` (`Object`'s identity comparison) where `say ('x' == .array)`
compares against the array's items joined.

The audit's non-operator cases split the same way: a `DO` header's `initial`/`TO`/`BY` reach
`callOperatorMethod(OPERATOR_PLUS)` and are receivers of a unary operator, while `exprr` and `exprf`
reach `requestString` and are `reqstr` rows. `Interp::accept_header_value` now carries that split at
the code.

`message_send_argument_object_not_a_string.rex` is not edited -- editing a corpus file reddens
`sourceline_matches_the_interpreter_for_every_corpus_program`, and its rows are still true.
`corpus/lang/required_string_method_argument.rex` is the same line re-derived: the audit asked which
value *shapes* have a string value, and the section says the question is a message send, so a
`makeString` puts an object on the answering side.

## What I tested

Every probe below was run from a fresh empty directory with absolute paths, three descriptors read
separately, on both engines, against
`( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -s KILL 10
/home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`.

### The headline

```
say .k
::CLASS K
::METHOD makeString CLASS
  return "K says hello"
```

At `df799fee0`: oracle rc 0 `K says hello`, crate rc 0 `The K class`, both engines. After
`b305019ec`: `MATCH` on both engines.

### The corpus programs

Nine, all in `corpus/phase-5a.txt`:

| program | what it pins |
|---|---|
| `required_string_contexts.rex` | every reachable context, `makeString` answering |
| `required_string_default_name.rex` | the same objects with no `makeString`: the ten contexts that can use any string answer, and nine that cannot raise 26.2/26.3/26.5/26.6/25.11/24.1/16.1/43.1 from the default name |
| `required_string_nostring.rex` | NOSTRING untrapped, trapped by name, trapped by `ANY`, disarmed again, and a `makeString` receiver as the control |
| `required_string_face.rex` | `~request`/`~string`/`~objectName`/`~objectName=` over every receiver kind, and the rename's reach |
| `required_string_builtin_arguments.rex` | position order, `SUBSTR`'s unconditionally fetched pad, and the object-versus-conversion naming |
| `required_string_operator_argument.rex` | every dyadic family, both error namings, and the right operand with no `makeString` |
| `required_string_method_argument.rex` | the "all other methods" rule, answering and raising |
| `required_string_make_string_raises.rex` | the `REQUEST` frame, one deep |
| `required_string_argument_make_string_raises.rex` | the same frame with the method's own on top |

The last two are in `RAW_STDERR_COMPARISON`: the frame line carries its own `*-*`, a blank line-number
field and a run of spaces, and those bytes are the subject.

### Both engines

Every probe quoted in this report was run on `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker`, and the
harness insisted the two agreed with each other before comparing either against the oracle. Beyond
that, `ir_dual` runs every corpus program on both engines and compares them to each other:
`REXX_CORPUS_GATE=1 cargo test --release --test ir_dual` is `9 passed; 0 failed`, where the 9 is that
file's test functions and the corpus population is one of them. `corpus.rs` compares the default
engine against the oracle, so the pair is what makes "both engines match the oracle" a measured
statement rather than an inference from one of them.

### The five gate commands

Run from `rust/`:

| command | result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, no diagnostics |
| `cargo test --release --workspace` | 98 `test result: ok` blocks, 0 failures |
| `REXX_CORPUS_GATE=1 cargo test --release --workspace` | `169 of 169 matching`, 0 failures |
| `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | 1832 passed, 0 failed |

The corpus was 160 and is 169.

Gate table C's `reqstr` row reads `agree`:

```
  agree          loud=no  5a   reqstr Required String Values            depth 1 parent xcremet provide.xml:723 reqstr.rex
```

### One flake, and it is not mine

The first `cargo test --release --workspace` of the final round reported
`builtin::datetime::tests::time_e_does_not_reset_the_anchor_time_r_does` failed with
`e2 = 0.004531, e3 = 0.005933`. It is an elapsed-time assertion and this machine was running several
of my own background test jobs at the time. Re-run on its own three times: `ok` each time. Re-run of
the whole command: `ok=98 failed=0`. It touches nothing this task changed.

## The two controls

Both were run against the release build, `--no-fail-fast`, over the whole workspace, so "nothing else
caught it" is measured rather than assumed. Each mutation was applied to `dispatch.rs`, the tree was
restored from a copy taken before the edit (never from git), and the restore was confirmed by
`git status --porcelain` showing no modification under `rust/crates`.

### Control 1 -- delete the `makeString` limb

The second of D50's two required controls. `Interp::make_string_or_none` is made to answer
`StringConversion::None` unconditionally, so the receiver's own behaviour is never asked for
`MAKESTRING` and the protocol falls straight to its last limb:

```rust
    fn make_string_or_none(&mut self, value: ObjRef) -> StringConversion {
        let _ = value;
        StringConversion::None
    }
```

`REXX_CORPUS_GATE=1 cargo test --release --test gate_table_c --no-fail-fast`:

```
  agree          loud=no  5a   unkno Defining an UNKNOWN Method         depth 1 parent xcremet provide.xml:526 unkno.rex
  diverge-stdout loud=no  5a   reqstr Required String Values            depth 1 parent xcremet provide.xml:723 reqstr.rex
```

**The `reqstr` row reddens** -- `diverge-stdout` where it reads `agree` at `HEAD` -- and the `unkno`
row beside it does not, which is what says the mutation is confined to this row's mechanism.

`REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast`: 96 `test result: ok` blocks and
2 `FAILED`. The two are `corpus_differential` (`161 of 169 matching`, so 8 programs disagree) and
`the_l0_subset_passes_again_under_collect_on_every_allocation`. The second is a real second
instrument: `required_string_make_string_raises.rex` no longer raises, so it runs on to allocate and
leaves the committed zero-collection set.

The 8 differing programs are the seven that use a `makeString` plus
`required_string_argument_make_string_raises.rex`. **`required_string_default_name.rex` is not among
them**, which is the pair working: it declares no `makeString`, so deleting the limb cannot change
it, and a build that answered it while failing the others is exactly what the pair separates.

### Control 2 -- table C's mutation 3, `makeString` with the wrong string

```rust
                match self.send_make_string(value)?.map(|_| self.text(b"wrong")) {
```

The `makeString` still runs -- its side effects still happen -- and its answer is replaced. On the
concept probe, three descriptors, both engines:

```
=== oracle rc=0
--- stdout
K says hello
request-with K says hello
request-without The NIL object
string The P class
untrapped The P class
nostring raised
--- stderr
=== crate/ir rc=0
DIVERGE
--- stdout
wrong
request-with K says hello
...
--- stderr
=== crate/tree-walker rc=0
DIVERGE
--- stdout
wrong
...
--- stderr
```

**rc 0 on both sides, empty stderr on both sides, differing on stdout alone** -- the silent-divergence
shape the row exists for. Gate table C reads `diverge-stdout` for `reqstr`.

`REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast`: 96 ok blocks, 2 FAILED, the same
two tests, `163 of 169 matching`, and the six differing programs are the six that use a `makeString`
in a language context:

```
  [UNCLASSIFIED] lang/required_string_contexts.rex: stdout, stderr, exit code differ
  [UNCLASSIFIED] lang/required_string_nostring.rex: stdout differ
  [UNCLASSIFIED] lang/required_string_face.rex: stdout differ
  [UNCLASSIFIED] lang/required_string_builtin_arguments.rex: stdout differ
  [UNCLASSIFIED] lang/required_string_operator_argument.rex: stdout differ
  [UNCLASSIFIED] lang/required_string_method_argument.rex: stdout, stderr, exit code differ
```

### What control 1 found in my own work

The first run of control 1 did not report a divergence, it **wedged**. Two of the new corpus programs
armed `SIGNAL ON SYNTAX` above the clauses they expect to answer and set the loop counter below them;
under the mutation one of those clauses raises instead, the handler reads an uninitialised `n`, and
`n = n + 1` raises 41.1 from inside the handler, which re-arms and goes round for ever. `6f7899514`
puts the counter first. A witness that hangs under the mutation it exists to catch is not a witness,
and nothing but running the control would have found it.

## The sitting

Three builds in one sitting: `pinned=bench-baselines/pinned/rexx-run-15a1ffa98`, `base` built from
`df799fee0`'s `rust/crates` in this same tree and target directory, and `head`. Seven axes, five
rounds, `--task 14 --commit 6f7899514`.

* the pin's `sha256sum` is `141c3fa968ea774655bc00f3e3b9f61cf1c4b738fd2793831d0055e94f1d074b`,
  matching `PINNED.md`;
* `git log --oneline 15a1ffa98..HEAD -- rust/crates rust/Cargo.toml rust/Cargo.lock Cargo.toml` lists
  only commits the ledger records plus this task's own, so the pin is not stale;
* the duplicate-key check from `bench-baselines/README.md` prints `0`;
* `base` and `head` were built from one tree under one path prefix, so code placement is removed
  rather than bounded, and `cmp` says the `head` binary the sitting ran is byte-identical to a rebuild
  at the final commit.

**This task's own contribution**, `instructions:u` per pass, `head` over `base` inside the sitting:

| axis | arm | base | head | delta | ratio |
|---|---|---|---|---|---|
| `alloc4c` | ir | 3603.686920 | 3610.685654 | +6.998734 | 1.001942 |
| `alloc4c` | tw | 5324.691078 | 5316.690634 | -8.000444 | 0.998497 |
| `arith` | ir | 24062.575220 | 23890.318964 | -172.256256 | 0.992841 |
| `arith` | tw | 26490.777996 | 26443.523304 | -47.254692 | 0.998216 |
| `compound` | ir | 1914.479906 | 1929.480073 | +15.000167 | 1.007835 |
| `compound` | tw | 2739.480016 | 2744.480208 | +5.000192 | 1.001825 |
| `dispatchclass` | ir | 6431.347343 | 6417.398606 | -13.948737 | 0.997831 |
| `dispatchclass` | tw | 6482.420950 | 6496.401787 | +13.980837 | 1.002157 |
| `emptyloop` | ir | 376.000016 | 373.000039 | -2.999977 | 0.992021 |
| `emptyloop` | tw | 437.999982 | 435.999986 | -1.999996 | 0.995434 |
| `strings` | ir | 5421.525375 | 5441.525377 | +20.000002 | 1.003689 |
| `strings` | tw | 9129.519432 | 9118.518591 | -11.000841 | 0.998795 |
| `varlookup` | ir | 871.000045 | 866.000008 | -5.000037 | 0.994259 |
| `varlookup` | tw | 1773.000019 | 1754.000159 | -18.999860 | 0.989284 |

**The widest regression is `compound`/ir at 1.007835, under the 1% line.** The widest movement of any
kind is `varlookup`/tw at 0.989284, and it is an improvement.

**Change or layout: what the sitting establishes is that layout moves it by about this much, which
makes the widest own contribution uninterpretable rather than clean.** `emptyloop.rex` is
`do i = 1 to n; nop; end`, and its per-pass path reaches nothing this task touched: no `SAY`, no
builtin call, no compound reference, no dyadic operator, and a controlled `DO`'s `initial`/`TO`/`BY`
are the arms of `accept_header_value` this task did **not** change -- the ones it did are `For`/
`OverFor` and `Count`, which serve `exprf` and `exprr`, and that program writes neither. Confirmed at
the code rather than by inspection of the program alone:
`grep -rn "arith_general\|apply_binary\|compare_values" crates/rexx-exec/src/run.rs` matches only
comments, so the loop machinery reaches no operator entry point either. That axis reads `pinned>base`
of exactly `1.000000` -- nothing between the pin and this task's base moved it at all -- and
`pinned>head` of `0.992022`.

**What that rules out and what it does not.** A change that added work could not make an untouched
axis go *down*, so the movement there is not added work; by elimination it is code placement. That
establishes **layout moves this sitting by up to about 0.8%**. It does **not** establish that
`compound`/ir's `+0.784%` is layout -- the two are the same order of magnitude and the sitting cannot
separate them. The honest reading is: the widest own contribution is under the 1% line, and the
sitting's own noise floor is close enough to it that the figure should not be read as a measurement of
work added. I did not run anything at an inlining site, so I make no claim about the mechanism.

**`varlookup` is not a second control, and saying so is the point.** Its loop writes `x = x + 1`,
which on the tree-walker arm reaches `Interp::arith_general_body` and therefore does pay one added
branch per pass -- and it reads `0.989284`, the widest movement in the sitting, in the *fast*
direction. So it is evidence in the same direction and not an independent control.

**The accumulated figure is context and not mine.** `pinned>head`, `instructions:u`. One row per
axis, and the row shown is the **highest `pinned>head` ratio** over both arms and both sizes -- highest
ratio and not furthest from 1.000000, because the guard's question is "made nothing slower" and the
slowest row is the one that answers it. The two projections disagree on `arith`, `emptyloop` and
`varlookup`, all three below 1.000000 on either reading, and all three furthest from one on their `ir`
arm: `arith` at `0.993016` (`large`; its `small` is `0.993066`), `emptyloop` at `0.992022` and
`varlookup` at `0.994260`, the last two identical across `small` and `large`. The `size` column is
carried because for two axes the highest-ratio row is `large`:

| axis | arm | size | `pinned>base` | `pinned>head` |
|---|---|---|---|---|
| `strings` | ir | small | 1.009685 | 1.013410 |
| `dispatchclass` | tw | large | 1.009958 | 1.012133 |
| `compound` | ir | large | 1.002618 | 1.010473 |
| `alloc4c` | ir | small | 1.002702 | 1.004805 |
| `arith` | tw | small | 1.001500 | 0.999778 |
| `varlookup` | tw | small | 1.006814 | 0.996025 |
| `emptyloop` | tw | small | 1.000000 | 0.995434 |

**A `cycles:u` figure from this bench is not a result.** The widest spread in the sitting is
`bench-baselines/phase-5a-arms.tsv:8528` -- `dispatchclass | pinned>base | across_builds | tw | small
| cycles:u`, an across-builds *ratio* and not any one build's own figure -- whose five rounds run from
`value_min` `0.585463` to `value_max` `1.794598` around a median of `1.044888`. The corresponding
`instructions:u` row is `1.009940 [1.001562..1.011797]`. The `cycles:u` rows are all in the TSV and
nothing above rests on one.

## Files changed

**Round 0's own set.** Each fix round's section lists what that round changed; this list is not the
task's total.

Source:

* `rust/crates/rexx-exec/src/dispatch.rs` -- the protocol, `Object~string`/`~request`/`~objectName`/
  `~objectName=`, `String~makeString`, `required_string_argument`, the in-crate refusal tests
* `rust/crates/rexx-exec/src/run.rs` -- `SAY`, `PUSH`/`QUEUE`, `INTERPRET`, `ADDRESS`, `NUMERIC`,
  `SIGNAL VALUE`, `TRACE VALUE`, the `DO` header's two count positions, the indirect variable lists,
  the `CALL (expr)` target, the NOSTRING arming route
* `rust/crates/rexx-exec/src/eval.rs` -- the dyadic operators' right operand, the `>C>` move, the
  `object_operand_tests` re-derivation
* `rust/crates/rexx-exec/src/stem.rs` -- substituted compound tails
* `rust/crates/rexx-exec/src/parse_template.rs` -- `PARSE`/`ARG`/`PULL`
* `rust/crates/rexx-exec/src/builtin.rs` and the seven files under `builtin/` -- `Args`, the
  conversion over the whole argument list, and the mechanical parameter change
* `rust/crates/rexx-exec/src/lib.rs` -- `Interp::reqstr_armed` and its two directive arming routes
* `rust/crates/rexx-exec/src/error.rs` -- `Raised::nostring`
* `rust/crates/rexx-exec/src/ir/drive.rs` -- the two ops that propagated with `?`
* `rust/crates/rexx-exec/src/ir/compile.rs` -- the adjacency invariant's own doc
* `rust/crates/rexx-exec/src/plan.rs` -- `tail_key`'s test call sites
* `rust/crates/rexx-classes/src/registry.rs` -- `object_name`/`set_object_name`
* `rust/crates/rexx-core/src/body.rs` -- `NativeObject::set_rendered`

Corpus and harnesses: nine `corpus/lang/required_string_*.rex`, their nine
`sourceline_oracle/*.txt`, `corpus/phase-5a.txt`, `tests/corpus.rs`, `tests/coverage.rs`,
`tests/collect_stress.rs`, `bench-baselines/phase-5a-arms.tsv`.

Not touched, and modified in the working tree by someone else while I worked:
`docs/superpowers/plans/2026-08-17-phase-5a.md` and `rust/corpus/oracle-crashes.txt`. Neither is in
any commit of mine.

## Self-review findings

Read the whole diff against `df799fee0` with fresh eyes. What I changed as a result:

1. **A comment enumerated the members of a repo set.** `Interp::classify_string_conversion`'s doc
   named "a string, a number and an array" as the primitives whose `primitiveMakeString` agrees with a
   native `MAKESTRING`. That is an enumeration of this crate's value kinds, which can change. Now it
   states the property instead. Same for `blame_request`'s "the two internal callers" and
   `required_string_answer`'s "Two callers want different things".
2. **A comment enumerated the C++ callers of `evaluateStringExpression`.** Named the set instead --
   "every instruction that evaluates a single string expression".
3. **A doc went stale inside the same task.** `StringConversion`'s comment still said an array's string
   value "has to be handed the built object" after the `Bytes` split made that false.
4. **Two corpus programs could hang under a mutation** -- see the controls above.

What I looked at and left:

* `Args::values_from` indexes rather than slicing safely. Its one caller reads position 2 onward after
  position 1 has already been found present, so the slice is in bounds; a second caller would have to
  re-establish that.
* `whole_number`'s `args.object(position).unwrap_or(value)` can never take the fallback, because the
  converted list has the same shape as the objects list. It is a defensive default rather than a
  reachable arm.
* `signal_to_value`'s `if converted == value { return ... }` reads awkwardly. It is there so the
  ordinary path renders once rather than twice, and the alternative is a second `to_text` per
  `SIGNAL VALUE` clause.

## Concerns

1. **The ordering residual I first recorded as unenumerable is enumerable, and enumerating it is what
   found the fix round's behaviour defect.** See the fix report below: the set is 81 `BUILTIN(x)`
   blocks in one file, every one fetches in non-decreasing position order, and exactly one position in
   the whole set is fetched raw. The ordering concern is closed as hypothetical; the raw-position one
   was live and is fixed.
2. **Three of the section's contexts are refusals, and the refusal is the only thing keeping them
   honest.** A command, an `ADDRESS` command and `OPTIONS` are Phase 7, Phase 7 and 5c. Their
   `reqstr` behaviour lands with the instruction, and if a later task implements one of those without
   calling the protocol the result is a silent wrong answer with no row anywhere -- the shape this task
   exists to remove. `a_reqstr_context_whose_instruction_is_another_phases_is_still_loud` names all
   three, so at least the refusal cannot be quietly dropped.
3. **`~request` for a class name whose `MAKE` method this phase has no code for is loud, and that is a
   decision rather than a gap I could not close.** Answering `.nil` would be a wrong answer where the
   oracle converts. The in-crate test is the only instrument, because the oracle does not share the
   refusal.
4. **`~objectName=` on a string or a number is loud** for the same reason: the oracle stores a name in
   the object's own variable pool and this crate has no pool there. A program that renames a string
   gets rc 120 where the oracle gets rc 0.
5. **`>C>` moved to the read, and the adjacency it now depends on is asserted for a different
   reason.** `assert_read_echoes_follow_their_load` requires every `Op::TraceRead` to sit immediately
   behind its `Op::Load`, which is what keeps the compound's two trace lines together now that they
   come from two places. I added that reason to the assertion's doc; the assertion itself was already
   there, so nothing new enforces it.
6. **The latch's debug check detects and does not protect, and it is debug-only.** A release build has
   nothing there: release correctness rests on the arming sites being complete. The check now tests
   both limbs' routes and both were proved live by inversion (see the fix report), but it compares
   bytes and must not allocate, and a later edit could weaken either without a test noticing -- the
   collect-stress list would catch a new allocation, and nothing would catch the comparison being
   weakened.
7. **String's arithmetic, comparison and concatenation methods** are the other half of the section's
   method rule and have no reachable site in this phase, so the fallback-to-`~string` behaviour is
   untested. The corpus program says so in its own comment rather than leaving the gap silent.

---

# Fix round 1

**Status: DONE_WITH_CONCERNS.** All seven findings addressed. Two were defects and both are fixed with
a witness; five were prose and are corrected against commands quoted below. The five gates are green,
the corpus grew from 169 to 170, and the fix round's own contribution is 1.000000 to six decimal
places on every axis but one, whose movement is smaller than the same axis's spread for a
byte-identical binary.

| commit | what |
|---|---|
| `13219ab92` | the raw argument position, the latch check's second route, the corpus program, the prose sweep |
| `1d87d90cc` | the fix round's own three-build sitting, and the byte-identity control it produced |

## Finding 1 -- `VALUE`'s new value is fetched raw, and the fix is the shape and not the call site

**Reproduced first.** Both witnesses, three descriptors, both engines, fresh empty directory:

```
r = value('a', .K); say 'after'; say 'stored' a     (makeString says 'K asked')
  oracle       rc 0  before / after / K asked / stored converted
  crate/ir     rc 0  before / K asked / after / stored converted
  crate/tw     rc 0  before / K asked / after / stored converted

r = value('a', .Array); say 'cls=' a~class          (an unrelated ::METHOD makeString arms the latch)
  oracle       rc 0  cls= The Class class
  crate/ir     rc 0  cls= The String class
  crate/tw     rc 0  cls= The String class

signal on any name h; r = value('a', .Array); say 'after'   (no makeString anywhere)
  oracle       rc 0  before / after
  crate/ir     rc 0  before / trapped NOSTRING The Array class
  crate/tw     rc 0  before / trapped NOSTRING The Array class
```

**The shape, settled by enumerating the whole set rather than by patching `VALUE`.**
`BuiltinFunctions.hpp:55`-`:86` gives a `BUILTIN(x)` body a converting family of argument accessor
(`required_string`, `optional_string`, `required_integer`, `optional_integer`,
`required_big_integer`, `optional_big_integer`, `optional_pad`) and a raw family (`get_arg`,
`optional_argument`, `arg_exists`, `arg_omitted`), the second being `stack->peek`. I parsed every
`BUILTIN(x)` block in `expression/BuiltinFunctions.cpp` for its own `x_<name> = N` position constants
and for which family names each, brace-matching the blocks:

```
BUILTIN blocks: 81
builtins with a position fetched ONLY through a raw accessor: 1
   VALUE [(2, 'newValue')]
```

and the same pass over fetch order:

```
builtins whose converting fetches are NOT in non-decreasing position order: 0
```

So: exactly one position in the whole set is never converted, and no builtin fetches out of position
order. `/bin/grep -rln "^BUILTIN(" --include=*.cpp .` from `interpreter/` names that one file, which is
what bounds the enumeration.

**The fix.** `crate::builtin::RAW_ARGUMENT_POSITIONS` is that result as a table,
`raw_argument_positions` reads it, and `Interp::required_string_arguments` skips those positions while
converting the rest in position order. The lookup happens **behind** the latch test, so an ordinary
builtin call still pays one bool and nothing else.

**The regression instrument, and it is the only one.**
`corpus/lang/required_string_builtin_raw_argument.rex` arms the latch, stores an object through
`VALUE`, and reads it back -- because the wrong answer is only visible once something renders the
stored value. **It also carries rows under a live NOSTRING trap, and fix round 2 corrects what those
rows witness**: they show a raw position not raising under a trap, and they say nothing about how the
latch got armed, because the program's own `makeString` directives arm it at install. Proved live:
with the exemption removed
(`static RAW_ARGUMENT_POSITIONS: &[(&[u8], &[usize])] = &[];`), the program differs on stdout **and**
on exit status, on both engines:

```
=== oracle rc=0                    === crate/ir rc=1
before                             before
after                              K asked
class The Class class              after
K asked                            class The String class
stored converted                   stored converted
...                                ...
b is a class 1                     NOSTRING raised where the oracle stores the object
```

Restored and re-verified `MATCH` on both engines. Beside it, `builtin::tests::
every_raw_argument_row_names_a_builtin_and_a_position_it_can_take` refuses a table row naming a
builtin that is not implemented or a position past that builtin's own maximum. **What that test adds
is not what this paragraph first claimed** -- see fix round 2's finding H: the corpus reddens for an
orphaned row too, and the case only that test can see is an *empty* table, which fix round 2 makes it
refuse.

**Two neighbouring positions checked, so the exemption is one position and not a family.**
`VALUE`'s selector, position 3, goes through `optional_string` and both sides ask it for a string:
`value('zz', 'stored', .S)` prints `S asked` on the oracle and on both engines. And the oracle's
convert-at-most-once property, which is why a builtin fetching one position twice is not a second
conversion: `requiredStringArg` and `optionalStringArg` write the converted string back with
`replace(position, newStr)` (`expression/ExpressionStack.cpp:154`, `:186`), and `XRANGE`, whose loop
passes over its arguments twice, prints `K asked` exactly once per call on both sides at rc 0.

**The falsified corpus comment is corrected.**
`required_string_builtin_arguments.rex`'s header said "An argument the builtin does not use is
converted anyway" as a property of builtin argument lists. It now says what it witnesses -- SUBSTR
fetches its pad through `optional_pad` unconditionally -- and says plainly that a position the oracle
fetches with `stack->peek` is never converted, pointing at the new program. **The report's "Two
structural consequences" paragraph was *not* corrected in this round, though this sentence originally
claimed it was**; fix round 2 corrects it, and two more sites carrying the same stale reading.

## Finding 3 -- the latch check now tests both limbs' routes, and both are proved live

**What the check is for, stated plainly.** A wrongly clear latch is a *wrong answer*, and it is wrong
in two independent ways: limb 1 can answer a different string than the value renders as, and limb 3
can raise where the fast path renders. The old check tested only the first.

`required_string_latch_holds` now opens with the same gate `required_string_dispatch` applies --
`self.trap_for(b"NOSTRING").is_some_and(|trap| !trap.call)` -- and answers `false` for it. A trap that
could take NOSTRING while the latch is clear is wrong for every rendered object, because
`exec_condition_trap` is what arms both.

**Both routes proved live by inverting each write in turn**, debug build, one file at a time, restored
from a copy taken before the edit and `touch`ed so cargo rebuilds:

*Inversion A -- `arm_reqstr_for` made a no-op.* `say .k` with a class-side `makeString`:

```
a/ir rc=101
  stderr: thread 'rexx-interp' (694107) panicked at crates/rexx-exec/src/dispatch.rs:1792:9:
          the required-string latch is off where the protocol would answer differently or raise
a/tree-walker rc=101   (identical)
```

and the trap probe still answered correctly under that inversion (`trapped NOSTRING`, rc 0, both
engines), which is what says the assert is not simply always firing.

*Inversion B -- the `NOSTRING`/`ANY` arming replaced by `if false && ...`.*
`signal on nostring name h; say .array`:

```
b/ir rc=101
  stderr: thread 'rexx-interp' (694501) panicked at crates/rexx-exec/src/dispatch.rs:1792:9:
          the required-string latch is off where the protocol would answer differently or raise
b/tree-walker rc=101   (identical)
```

and the `makeString` probe still answered `K says hello` at rc 0 on both engines under that inversion.

*And the fix was necessary, measured rather than argued.* With inversion B **plus** the new trap test
removed from the check (`if false && self.trap_for(...)`), the same program is a silent wrong answer:

```
b/ir  rc 0  stdout: The Array class|after|   stderr: (empty)
b/tw  rc 0  stdout: The Array class|after|   stderr: (empty)
oracle rc 0 stdout: trapped NOSTRING|
```

Restored; both probes answer correctly again.

**What the release build has.** Nothing. The check is under `debug_assert`, so it *detects* in a debug
build and protects nothing in a release one: release correctness rests entirely on the arming sites
being complete -- `Interp::arm_reqstr_for`, called from both directive installers, and
`exec_condition_trap`'s `NOSTRING`/`ANY` arm, with a `false` initialiser and no clear anywhere.
`Interp::reqstr_armed`'s doc now says that, and says which route is latent rather than covered: a class
inheriting a native `MAKESTRING` installs no `MAKESTRING` name, and `~new` is refused in this phase so
no such receiver exists.

**The two over-broad claims are corrected.** `reqstr_armed`'s doc no longer says an arming route added
without setting it "reddens the debug gate" without qualification, and concern 6 no longer says the
check is the only thing that would catch one.

## Finding 2 -- concern 1 replaced by the enumeration

The enumeration above is now concern 1's content: the ordering residual is hypothetical, because no
builtin that exists fetches out of position order, and the case that was live is the raw position,
which is fixed. **The claim I should not have written is "I did not enumerate them" about a set that
lives in one file and has 81 members.**

## Finding 4 -- the three counts, re-derived or deleted

| claim | command | result |
|---|---|---|
| "171 call sites name `to_text`" | `git grep -c 'to_text(' df799fee0 -- 'rust/crates/*'` summed | **149**, across 18 files (`git grep -l` \| `wc -l`). At the working tree, 158. The report now says 149 and names the command. |
| "82 signatures" | `git diff df799fee0 -- rust/crates \| grep -c '^-.*args: &\[Option<ObjRef>\]'` and the matching `grep -c "^+.*args: Args<'_>"` | **79** and **79**. The report now says 79 and quotes both halves rather than a head total. |
| "the three raises below" over four `SELECT` branches | reading the file | the header now names the *message shapes* it covers and no longer counts the raises. |

## Finding 5 -- the sitting section

* The accumulated table now carries the **`size` column**, and its row per axis is the widest
  `pinned>head` over both arms **and both sizes**. Two axes' widest rows are `large` and the previous
  projection had quoted their narrower `small` figures: `dispatchclass`/tw is `1.012133` (large) where
  `1.012110` (small) was shown, and the `arith` row is now `tw`/`small` `0.999778` rather than
  `ir`/`small`.
* The `cycles:u` sentence now names the row it quotes:
  `bench-baselines/phase-5a-arms.tsv:8528`, `dispatchclass | pinned>base | across_builds | tw | small
  | cycles:u`, median `1.044888` over `[0.585463..1.794598]` -- an across-builds ratio, not a build's
  own figure. The `instructions:u` row for the same key is `1.009940 [1.001562..1.011797]`.
* **The layout conclusion is weakened to what `emptyloop` supports.** It establishes that layout moves
  this sitting by up to about 0.8%, which makes a `+0.784%` own contribution *uninterpretable* rather
  than *clean*; it does not establish that `compound`/ir's figure is layout. The unverified inlining
  mechanism sentence is gone -- nothing was run at that site.

## Finding 6 -- the citation

`ExpressionStack::requiredStringArgument` does not exist:
`grep -rn "requiredStringArgument" --include=*.cpp --include=*.hpp .` from `interpreter/` matches
nothing. The functions are `ExpressionStack::requiredStringArg` (`expression/ExpressionStack.cpp:142`)
and `ExpressionStack::optionalStringArg` (`:167`), each calling `argument->requestString()` and writing
back with `replace(position, newStr)` at `:154` and `:186`. `Interp::required_string_arguments`'s doc
now cites those. The report body carried no such citation --
`grep -inE "stringarg|expressionstack|expression/"` over it matches nothing -- so there was one
instance and it is fixed.

## Finding 7 -- the set-size sweep

Swept the whole diff rather than the two named instances:
`git diff df799fee0 -- crates corpus | grep '^+' | grep -iE '\b(two|three|...) [a-z]+s\b'`, then read
each hit and kept the ones that are measurements or name a set fixed outside this repo (a measured
transcript's frames, `provide.xml`'s own example clauses, the protocol's limbs, a `DO` header's
numeric positions, a catalogue entry's substitutions). Changed, all naming a set the code can
enumerate and that can grow:

* `Interp::reqstr_armed` -- "The protocol has two arming routes" (finding 7's own instance);
* `Interp::parse_strings` -- "the four sources that build their own bytes" (the other);
* `required_string_face.rex` header and `phase-5a.txt` -- "the protocol's own four messages: ~request,
  ~string, ~objectName and ~objectName=", which named a size *and* enumerated;
* `coverage.rs` -- the same phrase in the subset comment;
* `phase-5a.txt` -- "The three properties of a builtin's argument list";
* `builtin.rs` -- "two families of argument accessor", now named rather than counted, and "the two
  readings" softened to "both readings";
* `collect_stress.rs` -- "the two programs whose whole purpose is...";
* `dispatch.rs` -- "the two shapes of `~request` and `~objectName=`".

## The minor items

* `6f7899514`'s message describes the corpus programs and not the doc-comment hunk it also carries.
  Accepted; amending a commit mid-stack is a rebase and the hunk is a comment.
* The `OPTIONS` refusal says `(Phase 5)` where the owner is 5c. Left as it is: the refusal predates
  this task, and narrowing a refusal's owner string is the owning task's change, not a `reqstr` fix.
* The context table's `DO` row now maps `exprr` to the `Count` arm and `exprf` to `For`/`OverFor`, so
  it and the sitting section say the same thing.
* The `ir_dual` sentence now says the 9 is that file's test functions, one of which is the corpus
  population.

## The gates, after the fix round

| command | result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo test --release --workspace` | 98 `test result: ok` blocks, 0 `FAILED` |
| `REXX_CORPUS_GATE=1 cargo test --release --workspace` | 98 ok, 0 `FAILED`, **170 of 170 matching** |
| `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | 1833 passed, 0 failed |

The corpus was 169 and is 170. The debug count is 1833 where it was 1832: the new
`every_raw_argument_row_names_a_builtin_and_a_position_it_can_take`.

## The fix round's sitting, and a byte-identity control it produced

Three builds in one sitting: the pin, `base` from `127fb74b8`'s `rust/crates` built in this tree, and
`head`. Seven axes, five rounds, `--task 14-fixround-1 --commit 13219ab92`. The duplicate-key check
prints `0`; the pin's `sha256sum` still matches `PINNED.md`.

Own contribution, `instructions:u` per pass, `head` over `base`:

| axis | arm | base | head | delta | ratio |
|---|---|---|---|---|---|
| `alloc4c` | ir | 3610.685758 | 3610.686284 | +0.000526 | 1.000000 |
| `alloc4c` | tw | 5316.689918 | 5316.691646 | +0.001728 | 1.000000 |
| `arith` | ir | 23890.315976 | 23890.316372 | +0.000396 | 1.000000 |
| `arith` | tw | 26443.521244 | 26443.524048 | +0.002804 | 1.000000 |
| `compound` | ir | 1929.480028 | 1929.479915 | -0.000113 | 1.000000 |
| `compound` | tw | 2744.480140 | 2744.480385 | +0.000245 | 1.000000 |
| `dispatchclass` | ir | 6404.359463 | 6417.383708 | +13.024245 | 1.002034 |
| `dispatchclass` | tw | 6496.425343 | 6496.357048 | -0.068295 | 0.999989 |
| `emptyloop` | ir | 373.000038 | 372.999995 | -0.000043 | 1.000000 |
| `emptyloop` | tw | 435.999980 | 436.000012 | +0.000032 | 1.000000 |
| `strings` | ir | 5441.525012 | 5441.525599 | +0.000587 | 1.000000 |
| `strings` | tw | 9118.518587 | 9118.518509 | -0.000078 | 1.000000 |
| `varlookup` | ir | 866.000049 | 866.000030 | -0.000019 | 1.000000 |
| `varlookup` | tw | 1754.000076 | 1754.000080 | +0.000004 | 1.000000 |

Thirteen of the fourteen rows are `1.000000`, which is the null result the fix round predicts: the
table lookup sits behind the latch, the new trap test is `debug_assert`-only, and everything else is a
comment.

**`dispatchclass`/ir's `+13.024245` is the axis's own spread, and this time there is a byte-identity
control for it rather than an argument.** The binary this sitting labels `base` and the binary the
first sitting labelled `head` are both builds of `127fb74b8` in this tree, and they are byte-identical
-- `sha256sum` `fd1e44db9616429adbabed4639c1f27d0e2afc5b563d70ae42444f6f8c5f6863` for each. That one
binary read `dispatchclass`/ir per-pass **6417.398606** in the first sitting and **6404.359463** in
this one, a difference of 13.039 instructions per pass -- already larger than the delta above.

**13.039 is the distance between two medians and is the loosest of the bounds this data supports; the
floor is the per-round span.** Fix round 2 corrects the figure. That same identical binary's own
rounds run `[6387.410016..6417.424106]` in `14-fixround-1 base` and `[6404.357939..6421.384423]` in
`14 head`, so its readings span **33.974407** instructions per pass across the two sittings --
0.53% of the axis -- and **30.014090** inside `14-fixround-1 base` alone. Each median also sits inside
the other's range. **So the resolution floor on this arm is at least 30 instructions per pass, not 13,
and no reading of it at that size is a measurement of work.**

The accumulated `pinned>head`, highest ratio per axis over both arms and both sizes, is unchanged from
the first sitting to six decimal places except `dispatchclass`/tw (`1.012128` against `1.012133`),
which is what "the fix round adds no work" looks like from the other direction:

| axis | arm | size | `pinned>base` | `pinned>head` |
|---|---|---|---|---|
| `strings` | ir | small | 1.013410 | 1.013410 |
| `dispatchclass` | tw | large | 1.012133 | 1.012128 |
| `compound` | ir | large | 1.010473 | 1.010473 |
| `alloc4c` | ir | small | 1.004806 | 1.004805 |
| `arith` | tw | small | 0.999778 | 0.999778 |
| `varlookup` | tw | small | 0.996025 | 0.996025 |
| `emptyloop` | tw | small | 0.995434 | 0.995434 |

## Concerns after this round

1. ~~**The raw-position table is a fact about the oracle held in the crate, and nothing re-derives
   it.**~~ **Withdrawn in fix round 2, which wrote the re-derivation.** The blocker I cited did not
   exist: `tests/gate_tables/orx.rs:79` already reads `interpreter/` at test time, with its own doc
   giving the justification, so the precedent was in this crate's suite the whole time. Round 2 adds
   `the_raw_argument_table_re_derives_from_the_oracles_own_source`.
2. **`dispatchclass` cannot resolve about 30 instructions per pass on its `ir` arm** (figure corrected
   in round 2; this section first said 13, which is the median-to-median distance rather than the
   floor). Established by byte identity above. That is a fact about the guard rather than about this
   task, and it means the first sitting's `dispatchclass`/ir reading (`-13.948737`) was inside the
   noise too.
3. The concerns from the first round stand except 1 and 6, which this round replaced.

---

# Fix round 2

**Status: DONE_WITH_CONCERNS.** Eight findings, two of them instruments and six prose. All addressed.
The corpus is 171 of 171, the five gates are green, and the round's own sitting is a null result to six
decimal places on every axis.

| commit | what |
|---|---|
| `04cc77f3d` | the route-B corpus witness, the raw-position re-derivation, the empty-table guard, the prose corrections |
| `e8717a8b7` | the round's own three-build sitting, and the header claim it corrected |

**Round 3 is documentation and carries no behaviour change.** It exists because this section was
missing: round 2's answer to its most severe finding lived in a commit message and nowhere in the
evidence artifact. Everything below was re-run at `04cc77f3d`, not carried over from that message.

## The harness that killed the previous session, fixed first

`cmp.sh` bounded the oracle with `ulimit -v 1048576` and `timeout -s KILL 10` and bounded the crate
with `timeout -s KILL 20` and **no memory bound at all**. A looping crate build therefore ran for
twenty seconds allocating without limit. Both sides are now bounded the same way:

```bash
( ulimit -v 1048576; REXX_ENGINE=$eng timeout -s KILL 10 "$CRATE" "$F" )
```

And rc 137 is `128 + SIGKILL`, which a `timeout -s KILL`, a `ulimit` kill and a kernel OOM all
produce, so the script now prints the elapsed seconds and the bound beside every 137 rather than
letting three events read as one. Self-tested on `do forever; end`:

```
=== oracle rc=137
    (rc 137 = 128+SIGKILL: the 10s timeout fired after 10s -- oracle)
=== crate/ir rc=137
    (rc 137 = 128+SIGKILL: the 10s timeout fired after 10s -- crate/ir)
MATCH
```

**The only rc 137 in this report is that self-test.** No measurement it rests on carries one, so
nothing above or below depends on telling the three causes apart -- but the label is there now for the
next reading that does.

## Finding A -- the route-B arming site had no corpus witness, and a committed comment said it did

### What was false

`corpus/lang/required_string_builtin_raw_argument.rex`'s header said its last rows were "the protocol
armed by a NOSTRING trap instead of by a makeString, which is the arming route that needs no directive
at all", and fix round 1's report repeated it. The program declares two `::method makeString`
directives, and `Interp::arm_reqstr_for` runs at directive install, before the first clause executes --
so route A had already set the latch by the time the `SIGNAL ON` ran.

Ruled by running rather than by reading. With the route-B arming site disabled --
`crates/rexx-exec/src/run.rs:3953` changed to `if false && matches!(&trap.condition[..], b"NOSTRING" |
b"ANY")`, release build -- that program **still matches the oracle** on both engines:

```
########## lang/required_string_builtin_raw_argument (trap-arming site DISABLED)
=== oracle rc=0
=== crate/ir rc=0
MATCH
=== crate/tree-walker rc=0
MATCH
```

Its header now says what those rows do witness -- a raw position not raising under a live trap -- and
says plainly that they say nothing about how the latch got armed.

### The witness that does cover it

`corpus/lang/required_string_nostring_no_directive.rex` **declares nothing**: `grep -cE "^::"` over it
is `0`. So the only thing in the file that can turn an object with no string value from a rendering
into a raise is the `SIGNAL ON` itself. It carries both spellings, because a trap table answers
NOSTRING through its own name and through `ANY` by separate lookups (`Interp::trap_for` consults the
condition's own key and falls back to `ANY`), and it ends on `VALUE`'s raw position under a live trap.

At `04cc77f3d` it matches:

```
=== oracle rc=0
--- stdout
untrapped The Array class
untrapped concat xThe Environment Directory
armed by name
by name NOSTRING | The Array class
disarmed The Array class
armed by any
by any NOSTRING | The Environment Directory
raw position stored 1
raw position is not a string 0
done
--- stderr
=== crate/ir rc=0
MATCH
=== crate/tree-walker rc=0
MATCH
```

**Proved live by disabling the arming site**, same mutation as above, release build. It diverges on
**stdout alone** -- rc 0 and empty stderr on both sides -- with each raise replaced by the rendering it
would have interrupted:

```
=== crate/ir rc=0
DIVERGE
--- stdout
untrapped The Array class
untrapped concat xThe Environment Directory
armed by name
not reached The Array class          <- the raise did not happen
by name  |                           <- so the handler reads no condition
disarmed The Array class
armed by any
not reached The Environment Directory
by any  |
raw position stored 1
raw position is not a string 0
done
--- stderr
=== crate/tree-walker rc=0   (identical)
```

**The contrast is what makes it a witness rather than a program that happens to be green.** Under the
same mutation, every other corpus program that arms a NOSTRING-taking trap still matches. The set is
derived rather than remembered: over the union of all four phase manifests -- 171 programs, counted by
`grep -vhE '^\s*#|^\s*$' phase-5a.txt phase-4a.txt phase-4b.txt phase-4c.txt | sort -u | wc -l` -- the
programs matching `signal on (nostring|any)` are exactly four, and with the arming site disabled:

| program | verdict, both engines |
|---|---|
| `lang/required_string_nostring_no_directive.rex` | **DIVERGE** |
| `lang/required_string_nostring.rex` | MATCH |
| `lang/required_string_builtin_raw_argument.rex` | MATCH |
| `lang/condition_nomethod.rex` | MATCH |

(`condition_nomethod.rex` also carries a `call on any`. `CALL ON NOSTRING` is a parse error, measured
in this task at 25.1 with `NOSTRING` named as the subkeyword it will not take, so the `CALL ON`
spelling cannot arm this condition by name; whether a `CALL ON ANY` can take it is not something I
measured, and `Interp::required_string_dispatch` excludes `call` traps for the reason
`Interp::novalue_raised` gives for its own condition.)

Restored afterwards: `git status --porcelain -- crates` clean, the rebuilt binary byte-identical to the
one the sitting measured, and the program `MATCH` on both engines again.

### What this check could not see

Had the claim been false -- had the program *not* depended on route B -- the mutation run would have
printed `MATCH`, which is exactly what the three other programs printed and what
`required_string_builtin_raw_argument.rex` printed before it was corrected. So the check discriminates,
and it is the same check that caught the false claim in the first place.

What it still cannot see: a route-B arming path that `exec_condition_trap` does not go through. The
mutation disables that one site, so a second site added elsewhere would leave this program green. The
`debug_assert` in `required_string_latch_holds` is what covers that, in a debug build only.

## Finding H -- the guard test's division of labour, corrected, and the empty table refused

Fix round 1's comment said "The corpus program is what catches the exemption being *wrong*; this
catches it being *absent*." Both halves were wrong: the corpus reddens for an orphaned row too, and the
genuinely absent case -- an empty table -- passed every arm of the guard vacuously.

`every_raw_argument_row_names_a_builtin_and_a_position_it_can_take` now refuses an empty table, and the
comment says what the test actually adds: it runs without the oracle, names the offending row, and
catches vacuity. Proved live, `RAW_ARGUMENT_POSITIONS` emptied:

```
test builtin::tests::every_raw_argument_row_names_a_builtin_and_a_position_it_can_take ... FAILED
RAW_ARGUMENT_POSITIONS is empty, which passes every arm below vacuously -- the oracle has at
least one raw argument position and `the_raw_argument_table_re_derives_from_the_oracles_own_source`
is what says which
test result: FAILED. 0 passed; 2 failed
```

Both guards fail there, which is the honest division: one says the table is empty, the other says what
it should have held.

## Finding 2's other half -- the raw-position table is re-derived, not recorded

Fix round 1 left the table as an enumeration run once by hand and recorded a concern that committing
the extractor "would need a decision about depending on the C++ tree at test time". **That blocker did
not exist.** `crates/rexx-exec/tests/gate_tables/orx.rs:79` returns a hardcoded
`/home/moritz/dev/repos/ooRexx/interpreter/RexxClasses`, scanned at test time, and its own doc gives
the justification. So the precedent was already running in this crate's suite, and the concern is
withdrawn rather than restated.

`builtin::tests::the_raw_argument_table_re_derives_from_the_oracles_own_source` now brace-matches every
`BUILTIN(x)` block in `expression/BuiltinFunctions.cpp`, reads each block's own position constants,
classifies each against the converting and raw accessor families, and asserts the derived set equals
`RAW_ARGUMENT_POSITIONS` restricted to the builtins this crate implements.

**It panics rather than skipping when the tree is absent**, which is D56's own rule about a check that
silently passes. Proved by pointing the path at a file that does not exist:

```
test builtin::tests::the_raw_argument_table_re_derives_from_the_oracles_own_source ... FAILED
cannot read /home/moritz/dev/repos/ooRexx/interpreter/expression/NoSuchFile.cpp -- the builtin
bodies this table derives from are part of [the read-only C++ tree...]
test result: FAILED. 0 passed; 1 failed
```

**It states its own blind spot and asserts it.** The derivation classifies the positions a block's own
`const size_t <NAME>_<arg> = N;` constant names. `MAX` and `MIN` hand their trailing positions straight
to `RexxString::Max`/`Min` as `stack->arguments(argcount - 1)` and name no constant, so those positions
are invisible to any such parse. They were ruled by running instead -- `max('1', .K)`, `max(1+0, .K)`
and `min('9', .K)` with a saying `makeString` each print `K asked` before the answer, oracle and both
engines byte-identical at rc 0, so they are converted and correctly absent from the table -- and the
set of blocks that do this is asserted against the source so a new member reddens rather than passing
unseen.

**Proved live three ways**, one mutation at a time, restored between each:

| mutation | output |
|---|---|
| `&[(b"VALUE", &[3])]` | `FAILED` ... `RAW_ARGUMENT_POSITIONS disagrees with .../BuiltinFunctions.cpp -- a position the oracle fetches raw and ...` |
| `&[(b"VALUE", &[2]), (b"LENGTH", &[1])]` | `FAILED`, the same assertion |
| `EXTRA_ARGUMENT_BLOCKS: &["MAX"]` | `FAILED` ... `the set of blocks that pass positions on without a constant has moved, and this ...` |

Restored: both guards `ok`, and `git status --porcelain -- crates` clean.

### What this check could not see

Had the table been right and the derivation wrong in the same direction -- a parse that missed the same
position the table misses -- the test would have passed. That is not hypothetical for the blind spot
above, which is why the blind-spot set is asserted separately: the derivation cannot classify those
positions, so the test says so out loud instead of quietly agreeing with itself. And a position reached
through a macro spelled differently from either family would be classified as neither and silently
excluded from both sides; the derivation does not currently assert that every accessor-shaped call it
sees belongs to a known family.

## The prose findings

| # | what | where |
|---|---|---|
| B | "converting the whole list is right" survived, in three places, after round 1 claimed it was corrected | report's context table, its "Two structural consequences" paragraph, and its corpus table; round 1's own sentence now records that it claimed a correction it had not made |
| C | the sweep corrupted `coverage.rs` into "the protocol's own / own messages" | rewritten as "the messages the protocol answers in its own right" |
| D | the shape relocated rather than removed | `reqstr_armed`'s "both directive installers" and `arm_reqstr_for`'s "the two installers", `Args`'s "both readings" in two places, `required_string_latch_holds`'s "two independent ways", and `required_string_operator_argument.rex`'s "the last two clauses", which was also loose about which clauses it meant |
| E | the accumulated table was labelled "widest" and projects highest ratio | relabelled, with the three axes where the two readings disagree named and their furthest-from-one figures given |
| F | the report head said 169 of 169 and listed a closed concern | says 171 of 171, and points at each round's own concern section as the live list |
| G | the "reddened eleven tests, which is the check working" stand-in | deleted; the surrounding paragraph now describes both limbs the check tests |

**One prose defect this round introduced and then caught.** The new program's header first said the
inversion made it diverge "on stdout and on exit status". It diverges on stdout alone -- both sides
rc 0 -- as the transcript above shows. Corrected. Editing that comment also invalidated the program's
committed `SOURCELINE()` expectation, which reddened
`sourceline_matches_the_interpreter_for_every_corpus_program` at `left: 58, right: 57` until the
expectation was regenerated; the file is 58 lines by `wc -l`.

## The gates, at the end of this round

| command | result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo test --release --workspace` | exit 0, 98 `test result: ok`, 0 `FAILED`, 0 `panicked` |
| `REXX_CORPUS_GATE=1 cargo test --release --workspace` | exit 0, 98 ok, 0 `FAILED`, 0 `panicked`, **171 of 171 matching** |
| `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | exit 0, 1834 passed, 0 failed, 0 `panicked` |

The corpus was 170 and is 171. The debug count is 1834 where round 1 left 1833: the new
`the_raw_argument_table_re_derives_from_the_oracles_own_source`.

## Round 2's own sitting

Three builds in one sitting -- the pin, `base` from `1d87d90cc` built in this tree, and `head` -- seven
axes, five rounds, `--task 14-fixround-2 --commit 04cc77f3d`. The duplicate-key check from
`bench-baselines/README.md` prints `0`. The `head` binary the sitting measured is byte-identical to a
rebuild after the mutation runs were restored.

**The first attempt at this sitting was killed with the session** and appended nothing --
`awk -F'\t' '$1=="14-fixround-2"'` over the TSV was `0` rows before the re-run and is `574` after, so
no partial sitting is in the file.

Own contribution, `instructions:u` per pass, `head` over `base`:

| axis | arm | base | head | delta | ratio |
|---|---|---|---|---|---|
| `alloc4c` | ir | 3610.686706 | 3610.685958 | -0.000748 | 1.000000 |
| `alloc4c` | tw | 5316.689526 | 5316.690436 | +0.000910 | 1.000000 |
| `arith` | ir | 23890.316436 | 23890.318020 | +0.001584 | 1.000000 |
| `arith` | tw | 26443.517464 | 26443.521852 | +0.004388 | 1.000000 |
| `compound` | ir | 1929.480028 | 1929.479951 | -0.000077 | 1.000000 |
| `compound` | tw | 2744.480108 | 2744.480080 | -0.000028 | 1.000000 |
| `dispatchclass` | ir | 6417.358006 | 6417.388813 | +0.030807 | 1.000005 |
| `dispatchclass` | tw | 6496.422019 | 6496.388434 | -0.033585 | 0.999995 |
| `emptyloop` | ir | 373.000028 | 373.000014 | -0.000014 | 1.000000 |
| `emptyloop` | tw | 435.999977 | 436.000031 | +0.000054 | 1.000000 |
| `strings` | ir | 5441.525727 | 5441.525787 | +0.000060 | 1.000000 |
| `strings` | tw | 9118.518319 | 9118.518675 | +0.000356 | 1.000000 |
| `varlookup` | ir | 866.000006 | 866.000011 | +0.000005 | 1.000000 |
| `varlookup` | tw | 1754.000019 | 1754.000052 | +0.000033 | 1.000000 |

**The widest movement in the whole sitting is 0.033585 instructions per pass**, on
`dispatchclass`/tw. Twelve of the fourteen rows are `1.000000` to six decimal places and the other two
are `1.000005` and `0.999995`. That is the null result the round predicts -- it adds one test, one
non-empty assertion, and comments -- and it lands three orders of magnitude below this axis's own
resolution floor, which the previous round measured at about 30 instructions per pass on the `ir` arm.

Accumulated `pinned>head`, highest ratio per axis over both arms and both sizes:

| axis | arm | size | `pinned>base` | `pinned>head` |
|---|---|---|---|---|
| `strings` | ir | small | 1.013410 | 1.013410 |
| `dispatchclass` | tw | large | 1.012135 | 1.012132 |
| `compound` | ir | large | 1.010473 | 1.010473 |
| `alloc4c` | ir | small | 1.004805 | 1.004805 |
| `arith` | tw | small | 0.999778 | 0.999778 |
| `varlookup` | tw | small | 0.996025 | 0.996025 |
| `emptyloop` | tw | small | 0.995434 | 0.995434 |

Every cell reproduces the previous round's to six decimal places except `dispatchclass`/tw, which
moves in the fifth, so the accumulated drift is where round 1 left it.

## Concerns after this round

1. **The re-derivation classifies against two named accessor families and excludes anything else
   silently.** A macro spelled differently from either list -- a new one upstream, or a renamed one --
   would be classified as neither, and both the derived set and the committed table would agree by
   omitting the same position. The blind-spot assertion covers the one shape known to exist
   (`stack->arguments`) and not that one.
2. **The route-B mutation disables one arming site.** A second site added elsewhere leaves the new
   corpus program green, and the `debug_assert` is then the only instrument -- in a debug build.
3. `dispatchclass`'s resolution floor on the `ir` arm is about 30 instructions per pass, which is a
   fact about the guard rather than about this task; it is stated here because that axis is now
   mandatory for later tasks.
4. The concerns from rounds 0 and 1 stand except round 1's concern 1, withdrawn above.
