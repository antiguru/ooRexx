# Phase 5c plan review

**Reviewed at** `414b71b22`. Read-only: nothing in the repository was edited. Everything below that
says "measured" has its command or transcript beside it.

**Subjects:** `docs/superpowers/specs/2026-09-02-phase-5c-class-methods.md` (D70–D74, M1–M9) and
`docs/superpowers/plans/2026-09-02-phase-5c.md` (seven tasks).

---

## The one-paragraph version

The plan's arithmetic is right and its citations to the 5b documents are right. Two things are
wrong at the level of the phase's premise rather than its details.

**First: gate table C's method rows do not measure method behaviour.** Every one of the 96 generated
programs is a `hasMethod` readback. Measured, all 118 String instance rows and all 47 TimeSpan
instance rows already answer byte-identically to the oracle on this crate — the first needs
`.String~new` to exist and nothing else, the second needs only `o = .TimeSpan~new(1)` and already
passes today. So the 452-row target is a **construction** target, and M2–M9's adapters and tables
move **zero** rows. D70 closed the narrow version of this hole ("match the refusal"); the wide
version is still open and is bigger.

**Second: the gate the plan prescribes is over 903 rows, not 452.** Run at BASE, exit 101, `868
row(s)` gated in table C and `35 row(s)` in table D. 371 of those sit on 15 classes that are
`covered` — so D70's criterion is blind to them — and that neither document names. The 35 in table D
are `::OPTIONS`, `::REQUIRES`, `::RESOURCE` and `::ROUTINE`; the plan mentions none of them.

---

## What I ran

* The C++ oracle, wrapped as the brief specifies, from fresh empty directories, three descriptors
  read separately: 40-odd probes across nine directories under
  `/tmp/.../scratchpad/planreview-5c/probe*`.
* `rust/target/release/rexx-run` at BASE (mtime 22:17:32, newer than every `.rs` in `rust/crates`)
  and `rust/bench-baselines/pinned/rexx-run-f558ea501`.
* `cargo test --locked --offline --release -p rexx-exec --test gate_table_c` in report mode, then
  the plan's own phase-gate command. `Cargo.lock` md5 was identical before and after every cargo
  invocation (`6da1507e7943d252b2222f8b19a48ef3`).
* Re-derivations of every number in both documents from `rust/corpus/docs/`.

---

# Findings, severity ordered

## F1 (critical). The 452 rows are a construction target. M2–M9 move none of them.

`gate_table_c.rs:824` `method_probe_text` emits, for the instance arm,

```
o = .{class}~new
say 'instance' o~hasMethod("...")     -- one per row
```

and for the class arm `say 'class' .{class}~hasMethod("...")`. Nothing else. Across all 96 programs
in `corpus/gate-tables/methods/`, every non-comment line is one of those two shapes:

```
$ cat rust/corpus/gate-tables/methods/*.rex \
  | /bin/grep -av '^\s*\*\|^/\*\|^\s*$\|hasMethod\|^o = \.\|^\s' | sort -u
(no output)
```

**A method is never sent.** So a row goes green when the name is in a dictionary and the constructor
answers.

Measured, all 118 String instance-arm `not-covered` names, asked of a literal `'abc'` receiver so
the missing constructor is out of the way:

```
$ ./target/release/rexx-run  p.rex   ->  rc=0, 118 lines
$ oracle                     p.rex   ->  rc=0, 118 lines
$ diff crate.out oracle.out          ->  IDENTICAL
$ sort -u crate.out                  ->  "instance 1"
```

and one row of it directly:

```
crate:  'abc'~hasMethod("ABBREV") ... ("?") ("[]") ("+")   ->  1 1 1 1 1 1   rc=0
crate:  'abc'~abbrev('a')                                  ->  rc=120  method "ABBREV" ... not implemented
oracle: same probe                                         ->  1 1 1 1 1 1 / 1   rc=0
```

TimeSpan is stronger still — it needs no implementation at all, only the construction program:

```
crate:  o = .TimeSpan~new(1) ; o~hasMethod(...) x47 ; o~totalSeconds
        ->  rc=0, 47 lines all "instance 1", then 0.000001
oracle: same    ->  rc=0, 47 lines all "instance 1", then 0.000001
        diff    ->  IDENTICAL
```

TimeSpan is 47 of Task 4's 70 rows, and it is done today except for one line.

**Consequence.** Task 2 (135 rows, five adapters, five tables), Task 3 (177), Task 4's TimeSpan half
and Task 5 can each be completed to the letter of their "done when" while moving no gate row, and
conversely all 452 rows can be turned green by writing 20 constructors and no adapter. Both criteria
1 and 2 are satisfied either way. This is [[gate-criteria-failure-modes]]'s "achievable and
worthless" at phase scale; D70 saw the version where the crate matches the oracle's refusal and did
not see this one.

**What the plan needs and does not have:** a criterion that a method *answers*. Nothing in gate
table C can supply it — the instrument that can is `corpus/phase-5c.txt` (see F9), which the plan
never asks anyone to create.

## F2 (critical). The prescribed gate is 903 rows over surfaces the plan does not scope.

Run exactly as `plans/2026-09-02-phase-5c.md:58` prescribes, at BASE:

```
$ REXX_PHASE_GATE=5c REXX_CORPUS_GATE=1 cargo test --locked --offline --release -p rexx-exec \
      --test gate_table_c --test gate_table_d --no-fail-fast -- --nocapture
exit=101
gate table C: mode: STRICT ... verdicts gated for 5a, 5b, 5c
              gated by this run: 868 row(s) ... whose verdict is not `agree`
gate table D: gated by this run: 35 row(s)
```

`METHOD_PHASE = "5c"` (`gate_table_c.rs:532`), so **every one of the 1347 method rows is 5c's**, not
only the 452. The report's verdict totals: `479 agree`, `497 unanswered`, `371 diverge-both`, summing
to 1347.

The 371 `diverge-both` rows sit on classes `class-set.txt` marks `covered`, verified one by one:

| Bag 28 | Directory 31 | IdentityTable 24 | List 38 | Properties 39 |
|---|---|---|---|---|
| **Queue 43** | **Relation 28** | **Set 24** | **Stem 27** | **Table 24** |
| **MutableBuffer 51** | **EventSemaphore 5** | **Monitor 4** | **MutexSemaphore 3** | **TraceObject 2** |

= 371, all rc 120 against an oracle at rc 0. Twelve fail on their own `NEW`; `Properties` fails on
`.Directory~new` and `Monitor` on `.Queue~new` (they subclass them), and `TraceObject` on
`a message send to one of the interpreter's own objects`. Because they are `covered`, **D70's
criterion 1 says nothing about them**, and because they are not in the 20, no task in the plan
touches them.

Gate table D's 35 are assigned at `gate_table_d.rs:251`:

```rust
("::OPTIONS" | "::RESOURCE" | "::REQUIRES" | "::ROUTINE", _) => Some("5c"),
```

The 35 red ones are **33 `::OPTIONS` rows plus `::REQUIRES LIBRARY` and `::REQUIRES NAMESPACE`**
(`::RESOURCE END`, `::ROUTINE PRIVATE` and `::ROUTINE PUBLIC` are also 5c's and already agree). Every
one is a rc-120 `rexx-exec: ::OPTIONS is not implemented (Phase 5)` or `::REQUIRES is not implemented
(Phase 5)` against an oracle that answers or raises its own error. Grep
for `OPTIONS|RESOURCE|NAMESPACE` in the plan: **0 hits**; in the spec: 1, and it is the
`directive-options.txt` row of the Authorities table.

Task 6's "Done when: every gate 0" therefore asks for work no task in the plan performs. Either the
scope is 903 rows and the plan is missing tasks, or criterion 2 has to be narrowed and say so.

## F3 (critical). D70's replacement criterion is gameable, and the tree already says how.

The brief asked whether D70's own criterion can be gamed. It can, in two independent ways, and
`gate_table_c.rs` names the second one itself.

**(a) The `covered` "committed construction program" arm does not exist in code.** The brief asked
Task 1 to find this out; the answer is available now. `status_of`
(`rexx-extract/src/docs/classes.rs:483`) reaches `Status::Covered` through exactly one path:

```rust
match construction {
    Some("new") => (Status::Covered, "a bare ~new constructs an instance on the oracle".into()),
    Some(code)  => (Status::NotCovered, format!("... a bare ~new raises {code} on the oracle")),
    None        => (Status::NotCovered, "... the name is not an .environment class entry".into()),
}
```

`CONSTRUCTION` (`:117`) is a hand-written const whose own doc says "**Measured, not derived**" and
"**Nothing re-measures this**, by decision". So `not-covered → covered` is moved by editing one
string literal from `"93.901"` to `"new"`, and `tests/extract_docs.rs` still passes because both
sides of its both-directions comparison read that same const.

**(b) Flipping the status does not change what the probe asks.** `method_probe_text` emits
`o = .{class}~new` unconditionally; `status` only selects the comment header. `gate_table_c.rs:815`
carries the obligation this creates, verbatim:

> **This derives only the first limb of `covered`.** … there is no route to a committed construction
> program because none exists to route to. The task that commits the first one has to add that route
> here in the same change; what it must not do is flip a class to `covered` and leave the derived
> probe on a bare `~new` that raises.

Task 1's brief (`plan:82-106`) names `classes.rs` and `tests/extract_docs.rs` and **does not name
`gate_table_c.rs`**, where half the work and the whole failure mode live. Its done-when — "the
`not-covered` count has fallen by construction alone, with the fall attributable per class" — is
satisfied by (a) alone.

## F4 (high). The M2/M5 split misassigns four class-arm rows, and `CNTRL` is not a BIF.

M2 claims "**49** of String's 132 documented **instance** names are BIFs the crate already
implements". Measured, four of the 49 are class-arm rows:

```
$ awk -F'\t' '$1=="String" && $4=="not-covered" && $3=="class"' corpus/docs/class-methods.txt
String  new    class  mthStringNew           fundclasses.xml:5152
String  alnum  class  mthStringAlnum         fundclasses.xml:5171
...  alpha 5192  blank 5213  cntrl 5233  cr 5252  digit 5271  graph 5291
String  lower  class  mthStringLowerClsMth   fundclasses.xml:5311
...  nl 5331  null 5348  print 5367  punct 5388
String  space  class  mthStringSpaceClsMth   fundclasses.xml:5406
...  tab 5427
String  upper  class  mthStringUpperClsMth   fundclasses.xml:5446
...  xdigit 5466
```

The block at `fundclasses.xml:5171-5466` is **sixteen** POSIX character-class methods, each titled
`(Class Method)` and each returning a character string. M5 lists twelve of them and hands `CNTRL`,
`LOWER`, `SPACE` and `UPPER` to the BIF adapter.

`CNTRL` is the hard error — there is no `CNTRL` BIF, on either instrument:

```
oracle:  say cntrl('abc')     -> rc=213  Error 43.1: Could not find routine "CNTRL".
oracle:  say c2x(.String~cntrl)
         -> rc=0  000102030405060708090A0B0C0D0E0F101112131415161718191A1B1C1D1E1F7F
```
```
$ comm -23 <M2's 49> <every name in corpus/builtin-status.txt>
CNTRL
```
(the other 48 are all `implemented` in that file.)

`LOWER`/`SPACE`/`UPPER` appear on *both* arms — they are 3 of the 3 rows by which 135 exceeds 132 —
and the class-arm one is not the BIF:

```
oracle:  .String~lower  ->  abcdefghijklmnopqrstuvwxyz
         .String~upper  ->  ABCDEFGHIJKLMNOPQRSTUVWXYZ
         c2x(.String~space) -> 090A0B0C0D20
```

**Root cause: the partition is keyed by name (132) while D71's unit is a row (135)** — the very
error M5 warns about for `BLANK`/`(BLANK)`, recurring four more times because the sum check is over
names. Keyed by `(class, name, arm)` the correct partition is

```
M2 48 (instance)  M3 34  M4 15  M5 16 (class)  M6 22   =  135 rows
```

and it checks on both arms independently: instance 48+34+15+21 = **118**, class 16+1(`NEW`) = **17**.

## F5 (high). Four of the 20 classes have no `~new` at all, and D73 does not cover them.

M1: "20 classes, one committed construction program each, arguments taken from the book's own syntax
for that class's constructor." Measured, four have no constructor to take arguments from:

```
.RexxContext~new        rc=163  93.967 NEW method is not supported for the RexxContext class.
.StackFrame~new         rc=163  93.967 NEW method is not supported for the StackFrame class.
.VariableReference~new  rc=163  93.967 NEW method is not supported for the VariableReference class.
.RexxInfo~new           rc=159  97.1   Object "a RexxInfo" does not understand message "NEW".
```

That is **57 rows** (RexxInfo 28, RexxContext 15, StackFrame 10, VariableReference 4). D73's
`unreachable` explicitly does **not** apply: `classes.rs:189`'s `UNCONSTRUCTIBLE` already carries all
four, classified `Status::NotCovered`, with the doc comment

> A row whose sentence says the user cannot construct one **and names a Rexx-level route** to obtain
> one makes a different claim; those are `not-covered`.

The routes exist and work, measured:

```
.context~class~id                 -> RexxContext        rc=0
.context~stackFrames[1]~class~id  -> StackFrame         rc=0
.RexxInfo~class~id                -> RexxInfo           rc=0
x = 5 ; say (>x)~class~id         -> VariableReference  rc=0
```

The plan has no bucket for this third case, and its Task 1 offers only "one documented as native-only
belongs under D73's guard instead", which is the classification `classes.rs` says is wrong for them.

**Related, and a smaller correction:** the plan says "**Three of the 20 have not been run against the
oracle at all** — the three comparators and `VariableReference`." `class-set.txt` already carries a
bespoke, re-read book sentence for `VariableReference` (`utilityclasses.xml:12556-12557`), unlike the
other 19. The three comparators genuinely had not been run; they are constructible with arguments:

```
.InvertingComparator~new       rc=163  93.901 Not enough arguments for method; 1 expected.
.ColumnComparator~new          rc=163  93.901 ... 2 expected.
.CaselessColumnComparator~new  rc=163  93.901 ... 2 expected.
```

So all three stay in M1, and open question 3 is answered: none is native-only.

## F6 (high). Task 0 cannot meet its done-when as scoped.

Task 0: "**Subject:** `.Array~of` and `.Directory~new`. Neither class is in the 20". Measured at BASE,
both engines:

```
alloc.rex      ir / tree-walker  rc=120  method "OF"  of class "Array"     not implemented
heapshape.rex  ir / tree-walker  rc=120  method "NEW" of class "Directory" not implemented
```

That much is right. But `alloc.rex:6-7` also needs `.string~new("item")`:

```
crate:  s = .string~new("item")  ->  rc=120  method "NEW" of class "String" not implemented
crate:  .array~new(3) / a~size / gc('F') / time('R'),'E'  ->  all rc=0
```

`String` **is** one of the 20, and `NEW` is one of M6's 22 — i.e. Task 2's. So "both programs are
rc 0" is not reachable inside Task 0's stated subject, and the whole "first, so the instrument exists
before the phase's bulk lands" rationale fails: the instrument needs the bulk.

`bench-programs/alloc4c.rex`'s own header, committed 2026-09-02, already records both refusals
verbatim, so this is a same-day contradiction inside the tree.

Second half of the same task: "`rexx-arms` reports a figure for each **against the pinned baseline**".
The pin cannot run either program —

```
$ ./bench-baselines/pinned/rexx-run-f558ea501 bench-programs/alloc.rex      -> rc=120 (Array OF)
$ ./bench-baselines/pinned/rexx-run-f558ea501 bench-programs/heapshape.rex  -> rc=120 (Array NEW)
```

— and `bench-baselines/phase-5b-arms.tsv` carries axes `alloc4c arith compound dispatch dispatchclass
emptyloop strings varlookup` and no `alloc`, no `heapshape`. There is no pin-relative figure to be
had for these two axes; what Task 0 can produce is an absolute sitting at head. The done-when should
say that.

## F7 (high). D71's only measured number is wrong in all three parts, at the commit it cites.

> "Measured at `2ca6e53f4`: **59 entries across 10 classes**, of which String has three (`LENGTH`,
> `REVERSE`, `UPPER`)."

Parsed out of `git show 2ca6e53f4:rust/crates/rexx-exec/src/dispatch.rs`:

```
NATIVE_METHODS: 79 entries, 11 classes
  Class 21  Object 19  Array 9  Package 6  StringTable 6  Directory 5
  String 4  Message 3  Method 3  Routine 2  RexxContext 1
  String: LENGTH, MAKESTRING, REVERSE, UPPER
NATIVE_CLASS_METHODS: 3 entries  (Object NEW, Array NEW, StringTable NEW)
```

79 is confirmed independently: 81 lines in the array span match `Arity::|A_`, two of which are
`A_COUNT` inside a comment.

**The more important half is what the number is missing.** Neither document mentions
`NATIVE_CLASS_METHODS` (`dispatch.rs:537`), which is where every class-arm row 5c lands goes — M5's
sixteen, M1's twenty `NEW`s, `String NEW`. It is a **separate array with a separate lookup
(`lookup_class_method`), separate panics, and no `.chain(extra)`**:

```rust
for (class_id, method_name, arity, run) in NATIVE_METHODS.iter().chain(extra) {   // :720
for (class_id, method_name, arity, run) in NATIVE_CLASS_METHODS {                 // :740
```

So D72's "the consumer already reads `NATIVE_METHODS.iter().chain(extra)`, so chaining more slices is
the shape that exists rather than a new one" holds for half the work. The class-arm half needs a seam
that does not exist. (D72's "two startup panics" is likewise four.)

**Also false, and load-bearing:** D71's "the table is data that `corpus/gate-tables/native-entries/`
already gates". That directory holds four programs (`file.rex`, `queue.rex`, `stream.rex`,
`timer.rex`) about `::METHOD ... EXTERNAL 'LIBRARY REXX stream_chars'` — the `NativeEntryPoint`
surface (`lib.rs:6868`), a different concept with a colliding name. `tests/native_entries.rs` never
mentions `NATIVE_METHODS`. Nothing gates `NATIVE_METHODS`' contents, which is the argument D71 uses
to justify a 452-row table as a small review surface.

## F8 (high). Tasks 3, 4 and 5 have no "Done when" clause.

```
$ /bin/grep -n "Done when\|^## Task" docs/superpowers/plans/2026-09-02-phase-5c.md
64:## Task 0 …   76:**Done when:**
82:## Task 1 …  105:**Done when:**
110:## Task 2 …  159:**Done when:**
165:## Task 3 — M7, reflection          (none)
176:## Task 4 — M8, time                (none)
186:## Task 5 — M9, collections and misc(none)
199:## Task 6 …  209:**Done when:**
```

317 of the 452 rows — 70% of the phase — sit in three tasks with no completion criterion at all.
Given F1, this is not a formality: a task with no done-when and an instrument that cannot see method
behaviour has nothing holding it to anything.

## F9 (medium-high). Task 6's control cannot fail, and `corpus/phase-5c.txt` is never asked for.

Task 6: "confirm `every_closed_phase_this_table_owns_rows_for_is_gated` … still passes. **Its control
is to remove `"5c"` and see the new state redden.**"

The assertion filters on the corpus subset file existing *before* it filters on `CLOSED_PHASES`
(`gate_table_c.rs:1450-1451`):

```rust
.filter(|phase| corpus.join(format!("phase-{phase}.txt")).is_file())
.filter(|phase| !CLOSED_PHASES.contains(phase))
```

`ls rust/corpus/phase-*.txt` → `4a 4b 4c 5a 5b`. No `phase-5c.txt`. So today's state is exactly the
state the control produces — `METHOD_PHASE = "5c"` owns rows, `"5c"` is not in `CLOSED_PHASES`
(`gate_tables/mod.rs:344` = `["5a", "5b"]`) — and it is green:

```
$ cargo test --locked --offline --release -p rexx-exec --test gate_table_c \
      every_closed_phase_this_table_owns_rows_for_is_gated -- --exact --nocapture
test every_closed_phase_this_table_owns_rows_for_is_gated ... ok
test result: ok. 1 passed; 0 failed
```

Removing `"5c"` after Task 6 adds it returns to precisely this state. The control reddens only if
`corpus/phase-5c.txt` is committed too, and the plan never says to create it. That file is also the
one instrument that could answer F1 — it is where a program that *calls* a method would live.

## F10 (medium). Tasks 3, 4 and 5 do share code, and Task 5 depends on Task 3.

"The order of Tasks 3, 4 and 5. **They share no code** and may run in parallel or in any order."

* Every one of them needs `NEW` in `NATIVE_CLASS_METHODS`, a three-entry array with no chain seam
  (F7). `.Directory~new` is rc 120 today precisely because it has no row there while `.Array~new`
  does, so a per-class row is required rather than inherited.
* Task 5 is told to write the D59a witnesses ("5c owes the witnesses"). D59a
  (`specs/2026-08-27-phase-5b-instances.md:999`) owes 5c two of them, and one is "`~subclasses` keeps
  counting a dropped class". `subclasses` is a **`Class` instance row**, i.e. Task 3's:
  ```
  $ awk -F'\t' '$1=="Class" && $4=="not-covered"' corpus/docs/class-methods.txt | grep subclasses
  Class   subclasses   instance   not-covered   ...
  ```
* Task 5's `CircularQueue` (47 of its 70 rows) is blocked on a class in nobody's task:
  ```
  crate:  o = .CircularQueue~new(5)  ->  rc=120  method "NEW" of class "Queue" not implemented
  ```
  `Queue` is `covered`, carries 43 `diverge-both` rows (F2), and appears in neither document.

## F11 (medium). M3's "reaches through `apply_binary`" is true for 24 of the 34.

`apply_binary` (`eval.rs:1693`) answers concatenation, comparisons and `And|Or|Xor`, and
`op => Err(Loud::binary_operator(op).into())` for everything else. Arithmetic takes a different arm
entirely (`eval.rs:569`, guarded by `is_arithmetic`). Against M3's list:

| reached by `apply_binary` | 18 comparisons + `& && \| \|\|` + `(abuttal)` `(blank)` | **24** |
|---|---|---|
| implemented, different path | `+ - * / // % **` | 7 |
| not a binary operator | `\` (prefix), `?`, `[]` | 3 |

The last three are not adapter work at all — they need bodies, i.e. they are M6's:

```
oracle:  (1 = 1)~?("apple","apples")  -> apple      -- two arguments
oracle:  "abc"[2] / "abc"[2,4]        -> b / bc     -- one or two arguments
book:    fundclasses.xml:5853-5855  "For NOT (prefix \), omit the parentheses and argument."
```

So M6's "the honest count of String's real work: 22 of 132" is at least 25, and M3's single adapter
has to become two.

## F12 (medium). The transposition control can be a witness that cannot fail.

Gate criterion 4 requires a transposition control "that reddens exactly its own row", and the plan
says "transpose one row's position". Measured, six of M2's BIFs are symmetric in the two arguments a
transposition would swap, so transposing them reddens nothing:

```
compare('abc','abd')=3  compare('abd','abc')=3
max(3,7)=7  max(7,3)=7      min(3,7)=3  min(7,3)=3
c2x(bitand('abc','abd'))=616260  c2x(bitand('abd','abc'))=616260
bitor and bitxor likewise identical
```

Seven of the eight receiver-position pairs the two documents quote *do* discriminate (F17), so a
control exists; the plan just has to say the chosen row must be one where transposition is
observable. As written it does not, and `COMPARE` is in the quoted eight.

## F13 (medium). Task 4's named instrument does not fit the use it is named for.

"If a documented behaviour cannot be compared deterministically, **record it as a licensed divergence
with its reason** … `tests/licensed_divergences.rs` is the existing instrument".

`LicensedDivergence` (`licensed_divergences.rs:91`) is `{ name, program, exit_code, stderr,
oracle_stdout, crate_stdout }` — **exact byte literals for both sides**, asserted with `assert_eq!`
against a live oracle run on every `cargo test`. It records a *deterministic* difference. A
non-deterministic `Alarm` or `Ticker` behaviour put there produces exactly the flaky row the plan is
trying to avoid. Adding a row also requires a matching DEVIATION row in
`plans/phase-4-exclusions.txt` (`the_prose_rows_and_this_table_name_the_same_divergences`), which the
plan does not mention.

## F14 (medium). D74's own precondition for starting 5c is not met at BASE.

D74: "`clause.rs`'s `Deadline` carries a 97-line doc comment … **Blocks of 48, 36 and 22 lines sit
beside it. 5c does not start until that file is brought under the policy**."

Doc-block line counts in `clause.rs`, before (`f9d0fb4c3`) and at BASE:

```
before:  97  48  36  30  22  22  22  21
BASE:        48  36  30  22  22  22  21
```

`964dc68ba` removed the 97-line block. The 48- and 36-line blocks are **byte-identical survivors**
(`diff` of both spans: no output). The surviving 48-liner is squarely what the policy bans:

> **Deliberately not `Copy` or `Clone` (fix round 4).** It was both, and that is what made
> `self.clause_state = <some other ClauseState>` … expressible from `run.rs` despite the private
> field, which round 3's module doc denied.

Either D74's sentence is stale and should say so, or the phase's stated precondition is open. The
plan offers `964dc68ba` as "the worked example of the trim" and says nothing either way.

## F15 (low). D70 quotes a transcript the program does not produce.

> "**while `.String~new('abc')~length~upper` remains unimplemented** — the oracle already answers
> that with `3 ABC`."

```
oracle:  say .String~new('abc')~length~upper                          -> 3        rc=0
oracle:  say .String~new('abc')~length .String~new('abc')~upper       -> 3 ABC    rc=0
```

The argument survives; the transcript belongs to a different program.

## F16 (low). D72's 2.42% sample has no committed record.

"Sampled at 2.42% of `bench-programs/dispatch.rex` under `hash_one::<&MethodId>`." Grepping
`docs/superpowers/` for `2.42` and `hash_one` finds neither together; the `2.42%` hits are
`alloc4c`'s wall-clock delta and an oracle spread, and the `hash_one` hits are
`2026-08-13-compound-name-resolution.md`'s `&[u8]` and `&SymbolId` figures. I did not re-measure it.
Under [[record-findings-where-the-project-looks]] a figure a later performance round is meant to act
on should be somewhere that round will look.

## F17 (low, and it is an omission not an error). The receiver-position table is right.

All eight pairs re-run, `s = 'abcabd'`, `w = 'abc abd abe'`, rc 0:

```
pos       meth=2       arg2=2       arg1=0
lastpos   meth=5       arg2=5       arg1=0
wordpos   meth=2       arg2=2       arg1=0
countstr  meth=1       arg2=1       arg1=0
changestr meth=ZbcZbd  arg2of3=ZbcZbd  arg1of3=a
abbrev    meth=1       arg1=1       arg2=0
verify    meth=6       arg1=6       arg2=0
compare   meth=6       arg1=6       arg2=6      <- symmetric, see F12
```

Neither document says what `s` was, so the transcript cannot be re-run from the document alone. Worth
one line.

## F18 (low). D61 binds 5c and neither document mentions it.

`specs/2026-08-27-phase-5b-instances.md:1029`: "**This governs 5c as well**: a row there may depend on
the class sweep's order and may not depend on the order over several [instances]." 5c's spec inherits
D57–D69 by reference but its risk table and Task 4/5 briefs — the two that touch collection and
`UNINIT` — do not carry it.

---

# What is sound

Verified and correct, so that a fix round does not re-litigate them:

* **Every count in the size table and the group table.** 1347 = 794 + 546 + 7; 23 classes carry a
  `not-covered` row; 5d's File 60 + Stream 25 + StreamSupplier 9 = 94; 546 − 94 = **452** over
  **20** classes. All twenty per-class counts in M1 are right and sum to 452. Reflection 177, time
  70, collections 70; 135 + 177 + 70 + 70 = 452. Re-derived from `class-methods.txt`, not read.
* **String's 135 / 132 / 118 instance / 17 class**, and that the three duplicates are `LOWER`,
  `SPACE`, `UPPER`.
* **The 49/34/15/12/22 partition is disjoint and is exactly the 132 names** — `comm` finds nothing on
  either side. Its defect is the arm assignment (F4), not the arithmetic.
* **D70's two-sets-of-23 claim, exactly as stated.** Run the report and compare:
  ```
  gate's unanswered set \ file's not-covered set  =  Pointer
  file's not-covered set \ gate's unanswered set  =  Singleton
  ```
  Both sets are size 23. The gate's unanswered row total is 497 against the file's 546, so the
  warning against quoting one for the other is well founded.
* **D70's four oracle transcripts.** `.String~new` 93.903 / `.String~new('abc')` `abc` rc 0 /
  `.WeakReference~new` 93.903 / `.TimeSpan~new` 93.901, all rc 163 as stated.
* **D73's inventory and citations.** Exactly seven `unreachable` rows — `Buffer new` and `Pointer`'s
  six — no other status on either class, and `utilityclasses.xml:429` and `:6910` both read "can only
  be created using the native code application programming interfaces." verbatim. Both refuse:
  `.Buffer~new` and `.Pointer~new` are 93.967. The sentences are already re-read on every run by
  `unconstructible_index`, so the guard has a live foundation.
* **M5's arm claim.** `'abc'~alnum` → `97.1 Object "abc" does not understand message "ALNUM"`,
  rc 159; `.String~alnum` → the POSIX string, rc 0.
* **The `BLANK` / `(BLANK)` / `(ABUTTAL)` three-row block** is quoted verbatim from
  `class-methods.txt`, and `fundclasses.xml:5919` is indeed
  `<section id="mthStringConcatenationMethods">`.
* **M4's sequencing.** Every one of the 15 `CASELESS*` names has its sibling inside the 132 — 7 in M2,
  8 in M6 — so "M4 last within Task 2" is the right order and nothing blocks.
* **Every citation into the 5b documents.** `plans/2026-08-27-phase-5b.md:730`, `:848`, `:944`,
  `:173`, and `specs/2026-08-27-phase-5b-instances.md:95`, `:621`, `:622` all say what is claimed,
  including "two of D59a's four consequences are this phase's".
* **`utilityclasses.xml:9956`** is `<section id="mthSupplierNew">`, so the 5d handover's reason for
  keeping `Supplier` in 5c is correctly cited.
* **`METHOD_PHASE` is already `"5c"`**, so `REXX_PHASE_GATE=5c` does reach the method rows — the
  problem there is scope (F2), not wiring.
* **`tests/extract_docs.rs` really does compare both directions**, and
  `tests/licensed_divergences.rs` really does assert its own table's completeness against the
  exclusions file. Both instruments are as described.

---

# What I did not check

* **I did not run the mutation that proves F3(a).** Showing that flipping one `CONSTRUCTION` string
  turns a class `covered` with nothing else changed requires editing a tracked file, and I am
  read-only. The finding rests on reading `status_of` and `CONSTRUCTION`, plus the tree's own
  statement of the same hazard at `gate_table_c.rs:815`. It should be run before the fix round
  closes.
* **F1's generalisation beyond String and TimeSpan.** I measured those two classes' full row sets on
  both sides. For the other eighteen the crate cannot construct an instance, so I could not ask
  `hasMethod` at all. The inference is from the probe shape, which is uniform across all 96 programs,
  and from the dictionaries being installed by `Interp::bootstrap_library` out of the same
  `CoreClasses.orx` the oracle uses. Not measured per class.
* **The five gates** (`fmt`, `clippy`, the three `cargo test` lines). I ran only `gate_table_c`,
  `gate_table_d` and one targeted test. Nothing here says the tree is otherwise green.
* **`Alarm` and `Ticker` construction.** `.Alarm~new` and `.Ticker~new` want two arguments and
  schedule work on a clock; I did not construct one, so Task 4's non-determinism claim is unverified
  by me. Its TimeSpan half is measured (F1).
* **`StreamSupplier`.** The handover's `.stream~new('t.txt')~supplier~class~id` transcript was not
  re-run; `class-set.txt` independently records the `97.1`.
* **D72's performance argument.** Neither the 2.42% sample (F16) nor the claim that an index into a
  `Vec<Option<NativeEntry>>` would be faster. `ObjectModel::build` is lazy per `Interp`
  (`dispatch.rs:1154`, `get_or_insert_with`) and runs a second time for
  `bootstrap_for_library`, so "iterated once, at startup" is approximate; I did not measure whether
  that matters.
* **Whether the 371 `diverge-both` rows were ever intended to be 5c's.** I established that
  `METHOD_PHASE` makes them 5c's and that the documents do not mention them. Which way that should be
  resolved — widen the plan or narrow criterion 2 — is a decision, not a finding.
