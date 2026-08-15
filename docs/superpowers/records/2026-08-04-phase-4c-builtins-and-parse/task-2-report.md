# Task 2 report: builtin dispatch, the arity table, and the 40.x error family

Branch `plan/rust-rewrite`, from `3a51a3b0`.

---

## 1. What was built

**New files**

* `rust/crates/rexx-exec/src/builtin/mod.rs` -- the name set, the arity rows,
  `is_builtin`, `dispatch`, `check_arity`, and five unit tests.
* `rust/crates/rexx-exec/src/builtin/string.rs` -- `LENGTH`, alone.

**Modified**

* `rust/crates/rexx-exec/src/run.rs` -- `resolve_and_run_call` restructured
  (below), a private `Resolved` enum, three new unit tests.
* `rust/crates/rexx-exec/src/lib.rs` -- `mod builtin;`.
* `rust/crates/rexx-exec/src/error.rs` -- `Raised::missing_argument` (40.5).
* `rust/corpus/builtin-status.txt` -- `LENGTH` `loud` -> `implemented`.
* `rust/crates/rexx-exec/src/eval.rs` -- **not in the brief's file list**; a
  4b test named `length` as its witness for the loud path and this work
  falsified it. See section 7.
* `rust/corpus/keyword-exempt.txt` -- **not in the brief's file list**; one
  ooTest body started passing. See section 7.

### The dispatch interface

The brief's signature is unchanged:

```rust
pub(crate) fn dispatch(
    interp: &mut Interp,
    name: &[u8],
    args: &[Option<ObjRef>],
) -> Option<Result<ObjRef, Failure>>
```

`None` means "not a builtin name". `is_builtin` and `dispatch` read the same
set, so `resolve_and_run_call` can use the first for resolution and the
second for the call without a second table.

### The name set

`is_builtin` answers from `rexx_inventory::builtins::in_scope()`, built once
into a `OnceLock<HashSet<&'static str>>`. Nothing is copied. `rexx-inventory`
was already a **regular** dependency of `rexx-exec` (not dev-only), so no
`Cargo.toml` change was needed.

`builtin::tests::the_partial_exclusions_are_builtin_names_and_the_whole_ones_are_not`
asserts the 63-vs-66 trap in both directions: every name in
`PARTIALLY_EXCLUDED` (`VALUE`, `ADDRESS`, `QUEUED`) *is* a builtin name here,
and every name in `wholly_excluded()` is not.

### Arity and implementation are one row

```rust
struct Builtin { name: &'static [u8], min: usize, max: Option<usize>, run: fn(..) }
const IMPLEMENTED: &[Builtin] = &[Builtin { name: b"LENGTH", min: 1, max: Some(1), run: string::length }];
```

`dispatch` runs `check_arity` from the same row it is about to call, so an
implementation cannot be entered with an argument list it did not ask for and
a builtin cannot be added without an arity. A builtin name with no row is a
declared gap.

### The restructured `resolve_and_run_call`

Order is now: build the activation body -> look up the label -> **resolve to
one of three outcomes** -> return `Loud::unresolved_call` for the third ->
evaluate the arguments **once** -> `Resolved::Builtin` returns through
`builtin::dispatch`, `Resolved::Label` falls through to `SIGL`, the depth
guard and the activation push exactly as before.

`Resolved` has two variants and not three: the "nothing" outcome returns at
the point of decision, so nothing downstream can hold a `Resolved` with
nothing to run.

---

## 2. Every oracle measurement

Wrapper used for all of them, from `.../scratchpad/t2probe-clean`, a directory
created empty for this task (checked: 2 entries, `.` and `..`; the obvious
name `probe/` already held 714 files from another agent):

```bash
( ulimit -v 1048576; \
  LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
  /home/moritz/dev/repos/ooRexx/build/bin/rexx <ABSOLUTE PATH> ) \
  > <ABS>.out 2> <ABS>.err
```

stdout, stderr and status were captured as three separate descriptors; the
status was read unpiped from `$?`.

### 2.1 The three rows the brief supplied -- all three reproduce exactly

| probe | rc | stderr |
|---|---|---|
| `say substr('abc')` | 216 | `Error 40.3:  Not enough arguments in invocation of SUBSTR; minimum expected is 2.` |
| `say substr('abc','x')` | 216 | `Error 40.12:  SUBSTR argument 2 must be a whole number; found "x".` |
| `say substr('abc',2,3,'pq')` | 216 | `Error 40.23:  SUBSTR argument 4 must be a single character; found "pq".` |

Each with the two lines above it, e.g.

```
     1 *-* say substr('abc')
Error 40 running /.../t2probe-clean/p01.rex line 1:  Incorrect call to routine.
Error 40.3:  Not enough arguments in invocation of SUBSTR; minimum expected is 2.
```

### 2.2 The further probing the brief asked for

**Too many arguments** -- `say length('abc','x')`, rc 216:

```
Error 40.4:  Too many arguments in invocation of LENGTH; maximum expected is 1.
```

**A missing required argument in a middle position** -- `say substr('abc',,2)`,
rc 216. **This is a sub-code the brief's table does not carry:**

```
Error 40.5:  Missing argument in invocation of SUBSTR; argument 2 is required.
```

**A negative where non-negative is required** -- `say substr('abc',2,-1)`,
**rc 163**, and it is *not* in the 40 family at all:

```
     1 *-* say substr('abc',2,-1)
Error 93 running /.../p06.rex line 1:  Incorrect call to method.
Error 93.923:  Invalid length argument specified; found "-1".
```

So a range violation on an argument that already converted to a whole number
is delivered by the underlying *method*, at 93.x and rc 163, not by the
routine-call check at 40.x and rc 216. A family task that assumes 40.something
for a negative length will ship the wrong number and the wrong exit code.

**The name is interpolated, not fixed** -- `say copies('abc','x')`, rc 216:

```
Error 40.12:  COPIES argument 2 must be a whole number; found "x".
```

**The name is upcased in the message while the `*-*` echo keeps the source
spelling** -- `say SuBsTr('abc','x')`, rc 216:

```
     1 *-* say SuBsTr('abc','x')
Error 40 running /.../p08.rex line 1:  Incorrect call to routine.
Error 40.12:  SUBSTR argument 2 must be a whole number; found "x".
```

Both confirmed in one transcript.

**`LENGTH`'s own arity** -- `say length()`, rc 216:

```
     1 *-* say length()
Error 40 running /.../p09.rex line 1:  Incorrect call to routine.
Error 40.3:  Not enough arguments in invocation of LENGTH; minimum expected is 1.
```

### 2.3 Trailing omitted arguments are not arguments -- found, not looked for

`say length('abc',)` prints **3** at rc 0, while `say length(,)` is 40.3 with
a minimum of 1. That is only consistent if a *trailing* omission is dropped
before the count is taken. Pinned directly with

```rexx
say '['q(1,,)']' '['q(,1)']' '['q(,)']' '['q()']' '['q(1,,2,,)']'
exit
q: return arg()
```

which prints `[1] [2] [0] [0] [3]`, rc 0. Interior omissions hold their place;
trailing ones are gone. `rexx-parse` already implements this
(`ExprKind::List`'s own doc comment, citing `parseArgList`'s `realcount`), so
an argument list arriving at `dispatch` has interior omissions only.

The consequence for `LENGTH` is that 40.5 is **unreachable from source** for
it -- a lone omitted argument is a trailing omission. `check_arity` still
implements the rule (it must decide the case rather than pass `None` into a
builtin), and its unit test drives it through a `SUBSTR`-shaped stand-in with
the measured substitutions.

### 2.4 The order of the checks

`say substr(,2,3,'p','q')` has both too many arguments *and* a missing
required first one. rc 216:

```
Error 40.4:  Too many arguments in invocation of SUBSTR; maximum expected is 4.
```

So the maximum is checked before anything about which positions were
supplied. `check_arity` applies max, then min, then required-position, in that
order.

`say substr('abc',2,)` prints `bc` at rc 0 -- the adjacent success: an
omission past the required positions is legal.

### 2.5 Which call forms reach the builtin table

| probe | rc | result |
|---|---|---|
| `say length('abc')` / `say Length('abcd')` | 0 | `3` / `4` |
| `say "LENGTH"('abc')` | 0 | `3` |
| `say "length"('abc')` | 213 | `Error 43.1:  Could not find routine "length".` |
| `call length 'abc'; say result` | 0 | `3` |
| `call "LENGTH" 'abc'; say result` | 0 | `3` |

A quoted literal target **does** reach the builtin table, matched on the
verbatim bytes with no folding -- the uppercase spelling resolves and the
lowercase one does not. This is why `dispatch` compares `name` against the
table's own upper-case spelling and upcases nothing.

### 2.6 A builtin beats a same-file `::routine` -- re-measured here

The claim was already in `resolve_and_run_call`'s doc from 4b. Restating a
measurement is a new claim, so it was re-run rather than quoted:

```rexx
call max 1,2
say result
say length('abcdef')
exit
::routine max
  return 'ROUTINE-WON'
::routine length
  return 'LENGTH-ROUTINE-WON'
```

rc 0, stdout `2` then `6`. Both builtins win. That is what makes the complete
66-name set load-bearing *before* Task 13 rather than after it.

### 2.7 `LENGTH`'s value model (D15)

```rexx
numeric digits 3
nn = length('abcdefghijklmnop')
numeric digits 1
say nn
say nn + 0
numeric digits 1
say length('abcdefghij')
```

rc 0, stdout `16`, `2E+1`, `10`.

Three separate facts. `nn` is created under `DIGITS 3` and read under
`DIGITS 1` -- the change D15 requires before a probe can see anything -- and
still prints `16`. `nn + 0` prints `2E+1`, because the addition is a new
operation creating a new number under the digits then in force; without that
line the first would also pass against a value that had merely captured
`DIGITS 3`. And `length('abcdefghij')` under `DIGITS 1` prints `10`, not
`1E+1`, which rules out creating the result through `Interp::number` with the
current settings at all.

So `LENGTH` creates its result as **text**, through `Interp::text` (which goes
through `Interp::alloc_with`), the same way `set_sigl` creates a line number.
A `number(16, digits=1, ..)` would render `2E+1` and be wrong.

Edge cases, rc 0: `say length('')` is `0`, `say length(123)` is `3`, `say
length(1.50)` is `4` -- bytes of the rendering, so a number argument is
measured by how it renders.

---

## 3. The three answers the brief demanded for the builtin path

All three measured, all three stated in `builtin/mod.rs`'s own module doc.

### 3.1 Is `SIGL` set? **No.**

```rexx
say 'sigl0=' sigl
n = length('abc')
say 'sigl1=' sigl
call sub
say 'sigl2=' sigl
exit
sub: return
```

rc 0, stdout:

```
sigl0= SIGL
sigl1= SIGL
sigl2= 4
```

`SIGL` is still its uninitialised derived name after the builtin call, and is
the `CALL`'s own line (4) after the label call. The builtin path therefore
returns *before* `set_sigl`.

### 3.2 Do `>A>` argument-trace lines fire? **Yes, identically to a label.**

`trace i` / `n = length('abc')`, rc 0, stderr:

```
     2 *-* n = length('abc')
       >L>   "abc"
       >A>   "abc"
       >F>   LENGTH => "3"
       >>>   "3"
       >=>   N <= "3"
```

against `trace i` / `n = sub('abc')` / `exit` / `sub: return arg(1)`:

```
     2 *-* n = sub('abc')
       >L>   "abc"
       >A>   "abc"
     4 *-*   sub:
     4 *-*   return arg(1)
       >L>     "1"
       >A>     "1"
       >F>     ARG => "abc"
       >>>     "abc"
       >F>   SUB => "abc"
       >>>   "abc"
       >=>   N <= "abc"
     3 *-* exit
```

Same `>L>`/`>A>` shape. This falls out of the restructure rather than being
arranged: the same argument loop serves both outcomes, and `>A>` is emitted
inside it. `>F>` comes from `eval.rs`'s existing `trace_intermediate` arm for
`ExprKind::Call`, and `>>>` from `exec_call`'s existing `trace_result` --
neither needed a change.

The `CALL` form, `trace i` / `call length 'abc'`, rc 0:

```
     2 *-* call length 'abc'
       >L>   "abc"
       >A>   "abc"
       >>>   "3"
```

No `>F>`, at the calling clause's own indent -- exactly what `exec_call`
already emits for a label callee.

### 3.3 Does the activation depth counter increment? **No.**

Two observables, both measured.

*Trace indent.* In the transcripts above, the builtin's `>F>`/`>>>` sit at the
calling clause's own indent, where the label callee's clauses (`4 *-*   sub:`)
echo two columns further in. `activation_indent` is derived from the push, so
no push means no elevation.

*Echo stack on a raise.* The oracle prints one clause echo per enclosing
activation. `say substr('abc')` at top level, rc 216, echoes **one** line.
Inside a routine:

```rexx
say 'start'
call sub
exit
sub: say substr('abc')
```

rc 216, stderr:

```
     4 *-*   say substr('abc')
     2 *-* call sub
Error 40 running /.../p19.rex line 4:  Incorrect call to routine.
Error 40.3:  Not enough arguments in invocation of SUBSTR; minimum expected is 2.
```

**Two** echoes -- the failing clause and the `call sub` -- and none for the
builtin itself. Inside a `DO` instead of a routine (`do i = 1 to 1; say
substr('abc'); end`) it is one echo again, at the `DO`'s indent. So the echo
count tracks activations, and the builtin contributes none.

Accordingly the builtin path returns before `MAX_ACTIVATION_DEPTH` and before
`self.activations.push`.

---

## 4. Task 1's Step 4.3 -- the falsification

`builtin/mod.rs` was copied to
`.../scratchpad/builtin-mod.rs.backup` (md5 `bd291cd9...`), then
`IMPLEMENTED`'s single row was replaced with `const IMPLEMENTED: &[Builtin] =
&[];` -- `LENGTH`'s dispatch arm and nothing else.

`cargo test --offline -p rexx-exec --test builtin_status`, **11 tests ran, 10
passed, 1 failed** (so this is not the "matched nothing, exited 0" false
green `rust/CLAUDE.md` warns about):

```
---- the_status_file_matches_a_live_differential_run stdout ----
rows whose measured status differs from .../corpus/builtin-status.txt:
  LENGTH: committed implemented, measured loud
        differing: [stdout, stderr, exit code]
        rust:   stdout="" stderr="rexx-exec: routine \"LENGTH\" is not implemented (4c)\n" exit=120
        oracle: stdout="6\n" stderr="" exit=0
```

**Exactly one row flipped, and it was `LENGTH`'s.** No other row moved in
either direction, so the arm's deletion is what the harness saw, not a
side-effect of the restructure. `every_loud_row_is_loud_about_its_own_builtin`
stayed green throughout, so the newly-loud row named `LENGTH` and not
something else. The build emitted one `dead_code` warning for
`string::length`, which is the expected shape of the mutation.

That is the falsification: the status file is derived from a live run of the
interpreter, not from a name table. A classifier consulting only names would
have gone on reporting `implemented`.

Restored with `cp` from the backup, **not** `git checkout --`. md5 of the
restored file equals the backup's (`bd291cd9...`), and `git status` afterwards
listed only the intended changes.

---

## 5. Commits

One commit, hash read back with `git log -1 --format=%H` after committing:

```
c10e40dfaaa0e723352a0f0183a7e430d067808c
Build the builtin dispatch, the arity checks, and LENGTH
```

Parent `3a51a3b0`. `git status --short` is empty afterwards.
`.superpowers/` is in `.gitignore` (`.gitignore:19`), so this report is not
part of the commit and does not leave the tree dirty.

---

## 6. The verify block

Run from `rust/`, each exit status read **unpiped**.

| command | status | result |
|---|---|---|
| `cargo test --offline --workspace --no-fail-fast` | **0** | 1,039 passed, 0 failed, 0 `test result: FAILED` lines across 71 result lines (baseline 1,031; +8 new tests) |
| `cargo fmt --all --check` | **0** | clean |
| `cargo clippy --offline --workspace --all-targets -- -D warnings` | **0** | clean |
| `REXX_CORPUS_GATE=1 cargo test --offline -p rexx-exec --test corpus` | **0** | 9 passed, 1 ignored |

`cargo fmt --all --check` failed once (status 1) mid-task on the new file and
on `run.rs`'s import order; `cargo fmt --all` was run and the check re-run to
0.

Because `rust/CLAUDE.md` records that a warm target directory can produce a
green clippy that never linted anything, the lint was **also** run against a
clean target directory:

```
CARGO_TARGET_DIR=<scratchpad>/clean-target \
  cargo clippy --offline --workspace --all-targets -- -D warnings
```

status **0** from a cold start.

---

## 7. Two files changed that the brief did not list

Both were forced by the work; neither was optional.

**`rust/crates/rexx-exec/src/eval.rs`.** 4b's
`a_builtin_name_still_fails_loudly_naming_4c` used `say length('abc')` as its
witness that an unresolved name is loud. Implementing `LENGTH` falsifies it
directly. Renamed to `an_unresolvable_name_still_fails_loudly_naming_4c` and
the witness changed to `zorkolo`, a name no table can hold -- a real builtin
here would assert where the implemented boundary sits and go red the day that
name landed, which is exactly what `corpus/builtin-status.txt` exists to carry
instead. No coverage lost: `builtin_status.rs` asserts the loud property over
every in-scope name and additionally that each loud row names its own builtin.

**`rust/corpus/keyword-exempt.txt`.** `keyword_assertions.rs` went red with
`NUMERIC::test_26 now PASSES but is still on the committed exempt list`. That
body is

```rexx
Numeric Digits 1
s=''
Do i=1 to 10
  s=s||i||'/'
  If length(s)>30 Then Leave
  End
self~assertSame(s, '1/2/3/4/5/6/7/8/9/1E+1/1E+1/1E+1/1E+1/')
```

-- `length` is its only builtin, and it passes now *because* the result is
DIGITS-independent, which is an independent confirmation of section 2.7. The
row was removed and the header's `4c  790 bodies` corrected to `789`. The
committed exempt set is 796 -> 795.

Nothing else in the tree moved. In particular, making `dispatch` recognise all
66 in-scope names (rather than only the implemented one) means arguments are
now evaluated before the loud exit for the 65 unimplemented ones; the full
`builtin_status.rs` run confirms **no** row's status changed as a result.

---

## 8. Things the brief got wrong, or did not know

1. **40.5 is missing from the brief's family.** An omitted argument in a
   middle position is `40.5 Missing argument in invocation of NAME; argument N
   is required.`, distinct from 40.3. Measured, section 2.2.

2. **A negative where non-negative is required is not in the 40 family.** It
   is `93.923` at **rc 163**, "Incorrect call to method". The brief lists it
   alongside the 40.x probes as though it belonged there. Measured, section
   2.2. Any family task implementing a length/position argument needs this.

3. **Trailing omitted arguments do not count.** Not mentioned anywhere in the
   brief, and it changes the arity model: the shared block's "An omitted
   position stays `None` rather than being closed up" is true only of
   *interior* omissions. Measured, section 2.3.

4. **A quoted-literal call target does reach the builtin table**, matched
   verbatim. `say "LENGTH"('abc')` is 3 and `say "length"('abc')` is 43.1.
   The brief's `search_labels = false` note reads as though a literal target
   resolves to nothing; it resolves to a builtin.

5. **The oracle evaluates a call's arguments before resolving the name at
   all.** `say zorkolo(1/0)` is `42.3` at rc 214, not 43.1 at 213. This crate
   keeps the loud return upstream of argument evaluation for a name that
   resolves to nothing, per the brief's Step 3 wording, so it answers a
   declared gap there rather than 42.3. That is a deviation on a path that
   is already a declared gap; **Task 13 owns it** and should move the loud
   return below the argument loop when it replaces the fallback.

6. **The 40.12 / 40.23 type raisers were deliberately not added.** The shared
   block assigns "the type family" to this task, but neither sub-code has a
   caller until a builtin with a typed argument lands: `LENGTH` accepts
   anything. Adding `Raised::not_whole_number`/`not_single_character` now
   would be `dead_code` in the library build and fail
   `clippy -- -D warnings`. The exact transcripts are in section 2.2 and are
   one line each to add beside a caller. 40.5 *is* added, because
   `check_arity` has to decide that case rather than hand a `None` to a
   builtin, so it has a live caller in shared machinery.

---

## 9. Commit hashes

`c10e40dfaaa0e723352a0f0183a7e430d067808c` -- "Build the builtin dispatch, the
arity checks, and LENGTH". Read back with `git log -1 --format="%H%n%s"`, not
written from memory. It is the only commit this task made.

## 10. Nothing was left undone

Every step in the brief was completed. The two deliberate departures are
recorded above and both are section 8: the 40.12/40.23 raisers are measured
and reported rather than added (no caller, `-D warnings`), and the loud return
for an unresolvable name stays upstream of argument evaluation per Step 3's
own wording even though the oracle evaluates first.
