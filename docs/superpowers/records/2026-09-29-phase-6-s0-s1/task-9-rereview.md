# Task 9 re-review -- fix round 1 (`8d65d9406..9e09cbc85`)

Reviewer: p6-t9-rereview. Built from `git archive 9e09cbc85` into `$R/tree` (touched),
`CARGO_TARGET_DIR=$R/target`, one `Compiling rexx-exec` line; `R` =
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/p6-t9-rr`.
`$R/cmp.sh FILE` runs the oracle (wrapped as CLAUDE.md says) and `rexx-run` from one fresh
`mktemp -d` in turn (same directory, so error paths match) and compares stdout, stderr and rc.
"pre" is the fix implementer's `p6-t9-fix/rexx-run.pre` (`8d65d9406`). Collect-every-allocation
runs use a scratch-only test (`tests/zz_rr_probe.rs` in the archive copy, never in the repo)
calling `run_program_collect_every_alloc` on `$PROBE`. Probes in `$R/probes/`. Gates not re-run.

## Verdicts

| finding | verdict |
|---|---|
| C1 | **Addressed** |
| I1 | **Addressed** |
| M1 | **Addressed** (as ruled: `collect_now` unit test) |
| M2 | **Addressed** (by C1) |

No new problem found in the round's own changes. Two pre-existing divergences found while
probing, identical on pre, listed at the end for queuing, not against this round.

## Check 1 -- `begin_init`: rooting, park/resume, INIT outcomes

- **Stack.** Scratch `zz_rr_stack` (same program shape as the round's drive test, `run_program`):
  `stack.bytes` is 0 at 50 and at 500 levels for Object (also with two args, one 11 bytes), List,
  Queue, Array, Directory, StringTable, Table, IdentityTable, Set, Bag, Relation, Class,
  MutableBuffer, WeakReference; `max_depth` 51 / 501, so the recursion ran. The report's
  "0 bytes for 50 and 500" holds.
- **Collection inside INIT.** `gc_init_inline.rex`: nine subclasses (Object, Array, Table,
  Directory, Class, MutableBuffer, WeakReference, Bag, Set), INIT arguments built inline by
  `copies` (20-byte strings rooted only by the call), INIT allocating 20 more 21-byte strings and
  storing a tag on `self`, read back after `~new` answers. Oracle-identical, rc 0; under
  collect-every-allocation identical stdout, 499 collections.
- **Park inside INIT.** `park_init.rex`: seven subclasses, INIT starts a `~start`ed method that
  allocates 400 strings, `call SysSleep 0.01`, then parks on `m~result`; arguments inline 20-byte
  strings. Oracle-identical; under collect-every-allocation three runs identical, 5762 collections
  each, every tag intact.
- **INIT RETURNs a value** (`init_return.rex`, 13 subclasses): `~new` answers the object each
  time; oracle-identical.
- **INIT raises, uncaught** (`raise_<class>.rex`, 13 classes): stdout, stderr (including
  `Compiled method "NEW" with scope "<Class>".`) and rc 214 identical for every class.
- **INIT raises, trapped inside INIT** (`init_trapped.rex`) and **trapped by the caller**
  (`trapped_outside.rex`: three different failing INITs under a routine's `SIGNAL ON SYNTAX`,
  each followed by further `~new`s at the same depth, so a stale `Then::Answer` tail would answer
  the wrong object): identical, rc 0.
- **INIT EXITs** (`exit_<class>.rex`, 13 classes; `init_exit_value.rex` with a value): EXIT ends
  the method, `~new` answers the object and the program continues, on both; identical.
- **Tree form** (`tree_new.rex`: `~new` beside a cascade, in a builtin argument, a failing INIT
  trapped by the caller): identical; collect-every-allocation identical.
- **Base classes with extra INIT arguments** (`extra_*.rex`: String, Stem, Message, Array,
  WeakReference, MutableBuffer, Directory, Table, Class, Package, Object): all identical on all
  three descriptors, including the INIT error frames.

## Check 2 -- String, Stem, Message subclasses

`sub_string.rex`, `sub_stem.rex`, `sub_message.rex`: post and pre both print `start` then
`rexx-exec: method "NEW" of class "K" is not implemented (Phase 9)` at rc 120; the oracle runs
INIT (rc 0). Refusal unchanged: same message, same rc, same stdout. `native_string_new` and
`native_stem_new` still refuse before allocating (`class != string/stem` ahead of `begin_init`).

## Check 3 -- class-specific NEW work before INIT

`classwork.rex` split into `cw1`-`cw7.rex`. Identical to the oracle: IdentityTable subclass with a
capacity; Array subclass `~new(2, 3)` dimensions and size, visible inside INIT (`6 2`) and after,
and `~new(0)`; MutableBuffer subclass buffer size 99 visible inside INIT; Class subclass id, class,
superclass and the 97.1 for the made class's `~new`; WeakReference subclass value inside INIT;
Directory and StringTable subclass key semantics. `direct_base.rex` (every changed class's
direct `~new`, Package from source included): identical, and identical to pre. `cw1` differs:
see "Pre-existing" below (identical on pre).

## Check 4 -- `new_init_subclasses.rex`

Oracle stdout read: every INIT prints three times per class (`aa`, `dd`, `tt` with its capacity
argument `10` then `5`, `ss`, `kk` with `extra` then `more`, `mb`, `wr`), each summary line
follows (`AA 1 0` ... `WR 1 0`), then the error. Stderr is `64 *-* return 1 / 0`,
`Compiled method "NEW" with scope "Table".`, `24 *-* b = .bad~new`, 42.3: the intended one.
Identical on all three descriptors. As the report says, at depth 3 it cannot tell stackless from
recursive; the drive and pinning tests do that.

## Check 5 -- I1 lent stack

`lend.rex`, each line beside a cascade so it evaluates through the tree: nested `f(g(4))` in a
builtin argument; two builtins each with a function-call argument in one expression; a compiled
builtin whose argument is a tree expression calling `f`; heap strings (18 bytes) as the lent
earlier arguments with callees that allocate and run builtins with arguments of their own; a
three-argument `translate` whose third argument calls a function; a `DO` header; a callee whose
SYNTAX is trapped by its own `SIGNAL ON`; a callee whose `CALL ON NOTREADY` handler runs builtins;
a callee using `INTERPRET`; a `::ROUTINE` callee. Oracle-identical (16 lines, rc 0);
collect-every-allocation identical, 80 collections. Pre fails the same file at line 2
(`5 1`, `8 1`, then 40.12 at rc 216), so the probe reaches the changed path. No aliasing seen:
`begin_call` builds its `CallTail` with `LentStack::Kept`, and `lend_stack` is called only from
`begin_invoke_call` and the compiled path, never twice for one tail.
Pre on `builtin_argument_call.rex`: `3`, `5 1`, `5 1`, then 40.3 on line 7, rc 216 -- the report's
"pre" line holds.

## Check 6 -- sentences

Read: the fix report, the four commit messages, the changed doc comments, the queue note
`2026-09-30-min-with-function-arg.md`. Checked against the tree: the remaining synchronous `INIT`
sends (`/bin/grep -a` over `dispatch/`) are `Array~of`, `Bag~of`, `Set~of`,
`native_collection_of`, `List~section`, `String~makeArray`, `array_of_texts`, `class_factory`,
`enhanced` -- none is a `NEW` row, and each is a `NativeBody::Run`, so it pins as
`Native(name)`. So `RESUMABLE_CLASS_METHODS`' "every `NEW` that sends `INIT`" holds. No false
sentence found.

Observation, not a finding: `corpus/oracle-crashes.txt` entry 21 names a `[]=` store inside INIT;
a read does it too -- `say 'init' self~items` inside a `Table` subclass's INIT is rc 139 on the
oracle (`seg_b.rex`), where a store after `~new` answers is fine (`seg_a.rex`). The entry is true
as far as it goes.

## Pre-existing (identical on pre; for the queue, not this round)

1. **A hash-collection subclass whose INIT does not forward to super keeps NEW's capacity.**
   `cw1.rex`: `.kt~new(100)` (a Table subclass with its own INIT) iterates in the order of the
   default capacity on the oracle and of capacity 100 here. The oracle sizes in `INIT`
   (`HashCollection::initRexx`), this crate in `NEW`.
2. **Same root: a StringTable subclass's NEW refuses a non-length first argument.**
   `st_args_sub.rex`: `.ks~new('abcdefghij', 'q')` runs INIT (rc 0) on the oracle and is 93.923 at
   rc 163 here. Base `.stringtable~new('x')` / `.directory~new('x')` agree on 93.923 and rc but
   the oracle's traceback has an extra `Compiled method "INIT" with scope "StringTable"` /
   `"Directory"` line (stdout same, stderr differs).
