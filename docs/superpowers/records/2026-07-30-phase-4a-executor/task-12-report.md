STATUS: DONE

# Task 12 report: errors, the message catalogue, and the exit code

Building the reporting subsystem in `rexx-exec/src/error.rs`. Task 7 built
`Raised` as a payload with propagation and `From<ArithError>`; the catalogue,
the two-line stderr format, the clause echo and the exit-code rule are this
task's, because they need oracle-captured expectations nobody had taken.

## Constraints carried into this

* **Text comes from `rexx-inventory`'s generated table** (704 messages from
  `rexxmsg.xml`). Hand-transcribing what the tree already generates is the
  thing a spec revision was made to prevent.
* **The format is exact**: clause echo with trace off, then two lines, two
  spaces after each colon, absolute path.
* **`exit code = 256 - major`**, confirmed for majors 7, 41, 42, 26 and 24.
* **The raiser families are an open list.** The plan's version is incomplete by
  construction, so 4a's surface gets walked rather than trusted.

Scope note: `eval.rs` has live Task 8 work, so this keeps to `error.rs` and
flags anything it needs elsewhere rather than reaching for it.

(Appended as each step completes.)

## Step 1: the raiser families, walked rather than trusted

Every family run under the oracle, wrapped. All fifteen the plan listed are
confirmed, including 26.2 and 26.3 which it carried as reported-but-unconfirmed:

| construct | error | rc |
|---|---|---|
| `select` with no true `WHEN`, no `OTHERWISE` | 7.3 | 249 |
| `if 'x' then` | 34.1 | 222 |
| `when 2 then` | 34.2 | 222 |
| `do while 'q'` | 34.3 | 222 |
| `do until 'q'` | 34.4 | 222 |
| `if 1=1, 'z' then` | 34.6 | 222 |
| `if 1 & 'p' then` | 34.901 | 222 |
| `numeric digits 'x'` | 26.5 | 230 |
| `numeric fuzz 'x'` | 26.6 | 230 |
| `numeric fuzz 9` then `numeric digits 5` | 33.1 | 223 |
| `say 'y' + 1` | 41.1 | 215 |
| `say 1/0` | 42.3 | 214 |
| `say 2**'x'` | 26.8 | 230 |
| `do i over .nil` | 98.913 | 158 |
| `trace 5` | 24.901 | 232 |
| `do 'x'` | **26.2** | 230 |
| `do i=1 to 3 for 'x'` | **26.3** | 230 |

### Three raisers no document lists

* **25.11**, `numeric form 'x'` -> rc 231. "NUMERIC FORM must be followed by
  one of the keywords SCIENTIFIC or ENGINEERING; found "&1"." **Major 25 is a
  family the plan's list does not contain at all**, so this is the same shape
  as the major-33 omission an earlier draft had.
* **24.1**, `trace value 'zz'` -> rc 232. "TRACE request letter must be one of
  "ACEFILNOR"; found "&1"." Same major as the known 24.901, different sub, and
  reachable from `Trace::Setting` as well as `Trace::Value`.
* **42.901**, overflow (`say 9e999999998 * 9e999999998`) -> rc 214.
  "Arithmetic overflow; exponent ("&1") exceeds &2 digits."

### Two negatives worth recording

* **`exit 'x'` does not raise.** rc 0, no error. An implementer would
  reasonably expect a non-numeric `EXIT` result to be an arithmetic
  conversion error; it is not.
* **`2 ** 999999999` does not overflow.** rc 0.

Also measured: `do i=1 to 'y'`, `do i='x' to 3` and `do i=1 by 'z' to 3` are
all 41.1, and `numeric digits 0` / `-1` are 26.5 rather than a distinct
sub-number.

## Step 2 and 3: the format, captured with `cat -A`

```
     4 *-* end$
Error 7 running /abs/.../f.rex line 4:  WHEN or OTHERWISE expected.$
Error 7.3:  All WHEN expressions of SELECT are false; OTHERWISE expected.$
```

Three things measured rather than inferred:

* **The clause echo appears with trace off.** It is not trace output, and
  `TRACE OFF` does not suppress it.
* **The line number is right-aligned in a six-character field** -- `     4`,
  `    12`, `   105`, checked at one, two and three digits.
* **The echo keeps its trailing space.** `if 'x' then nop` echoes `if 'x' `,
  because an `IF`'s clause span stops at the start of the token that ended its
  condition. Trimming it would diverge on every `IF`.

`say 1` went to stdout and all three lines to stderr, confirming the split
D17 relies on.

The text is looked up in `rexx-inventory`'s generated table. `lookup(major, 0)`
is the major line's text and `lookup(major, sub)` the sub line's, verified
against the oracle for majors 7, 24, 25, 26, 33, 34, 41, 42 and 98.

## Tests, and defeating the mechanism

Eight tests in `error.rs`. Verified they pin the format rather than merely
passing: changing two spaces after the colon to one, and the field width from
six to five, fails **three** of them; reverting restores 8/8.

The catalogue test deliberately asserts entries **exist** for all twenty
measured `(major, sub)` pairs rather than what they say, so it cannot drift
from `rexxmsg.xml` the way an assertion on the text would.

## What is not wired, and why

`ClauseSite`, `exit_code`, `report` and `substitute` carry
`#[allow(dead_code, reason = ...)]`. The caller is `execute()` in `lib.rs`,
which has live Task 8 work in the same crate, so wiring it is a change outside
this task's file. **What is needed: `execute` catching a `Failure::Raised`,
resolving the failing instruction's `clause_span`, line and absolute path into
a `ClauseSite`, and using `report`/`exit_code` instead of the loud path.**

## A mistake I made, and the lesson

`cargo fmt -p rexx-exec` is **package-wide, not file-scoped**, so it
reformatted `eval.rs` -- Task 8's live, uncommitted file -- as well as mine. I
did not commit it, and formatting cannot change behaviour, but another agent's
working tree now carries whitespace changes it did not make.

Use `rustfmt <path>` for a single file when other agents are live in the same
crate. `cargo fmt -p` is the wrong granularity for a shared worktree.

## Verification

* `cargo test -p rexx-exec --lib error::` -> **8 passed, 0 failed**.
* `cargo clippy -p rexx-exec --all-targets -- -D warnings` -> no findings in
  `error.rs`; the two remaining are in Task 8's `eval.rs`.
* `cargo build -p rexx-exec` -> clean.
* One test fails crate-wide,
  `eval::tests::a_comparison_reuses_an_already_parsed_num_cache`, which is
  Task 8's own in-progress test in its own file.

Commit `eb9eb0c8`, two paths: `src/error.rs` and `Cargo.toml`.
