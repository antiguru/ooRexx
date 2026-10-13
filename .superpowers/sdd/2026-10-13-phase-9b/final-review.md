# Phase 9b final whole-branch review

Range `f9105f68f..43a39ab01`, the source diffs read file by file
(`git diff --stat f9105f68f..43a39ab01 -- rust/crates`). I ran this as a mutation hunt across tasks. I
did not re-review each task on its own. The probes ran on a `git archive` copy of `43a39ab01` under
`/tmp/p9b-final` with its own `CARGO_TARGET_DIR`, and the copy was deleted afterwards. That copy had
one extra test file, `tests/zz_probe.rs`, which runs a program once normally and once under
`run_program_collect_every_alloc` in a fresh directory. Interpreters ran under `memcap 4G`. The
oracle ran under the standard wrapper, each probe from a fresh empty directory.

## Verdict

**Needs one fix (I1) before the close re-run.** Everything else is Minor. Each Minor item is either
a missing witness or record, or a lossy-path site that the queued file already covers in kind.

## Findings

### I1 (Important): a stem resolved by name is held unrooted across user code

`SysStemCopy`, `SysStemDelete`, `SysStemInsert` and `SysStemSort`.

`stem_argument` (`builtin/rexxutil/stems.rs:48`) and `sort_target` (`:542`, called at `:581`) take
the stem from `context_stem_named`. After that the stem is rooted only by the caller's variable.
More arguments are then converted (`optional_positive`/`positive` call `required_string_or_nil`, so
a `STRING` method runs), and `SysStemSort` also converts each element. If a `STRING` method drops
that variable and allocates, the stem can be collected while the routine is still using it.

Probes (`'s.'` passed by name, `self` as a later argument or element, its `STRING` doing
`expose s.; drop s.` and then allocating):

| routine | normal run | collect on every allocation |
|---|---|---|
| copy | `0 3` | rc 216, 40.922 |
| insert | `0 S.0` | rc 216, 40.922 |
| delete | `0 S.0` | rc 216, 40.922 |
| sort | `0 S.0` | rc 216, 40.924 |

- **The fix was checked in the copy.** Adding `push_temp(stem)` after both resolutions makes all
  four stress runs match the normal runs.
- **The oracle shares the defect.** With no allocation in the `STRING` method it answers `0 3`.
  With 200000 allocations it answers 40.922, 3 runs of 3. With the same allocations and no
  `drop`, it answers `0 3`. So the oracle's answer depends on when it collects: an undefined
  answer of the entry 39/40/45 kind.
- **This is not the licensed GC-ordering divergence.** That licence covers when a collection
  happens. It does not cover a change in rc or stdout, and that is what this is.
- **The same class as the Task 7 `security_entry` escape.** `collect_stress` cannot see it because
  no witness drops a stem while one of these routines is running.

Wanted:
- the push_temp at both sites;
- a corpus or crate-only witness with stem values longer than 7 bytes;
- an oracle-crashes entry recording that the oracle's answer depends on when it collects.

### M1 (Minor): two roots that matter but that no test exercises

I deleted four 9b-added roots in the copy, all at once, and ran `collect_stress` and
`cargo test --lib -- rexxutil`:

- The `security_check` root is caught (`requires_manager_rename.rex`).
- The dump's per-tail root is caught (`a_dumped_stem_keeps_its_tails_when_a_string_method_empties_it`).
- **Two deletions survive both binaries:**
  - **`SysDumpVariables`' `variables` root** (`stems.rs:727-729`). A probe whose `STRING` method
    drops two exposed heap-string variables panics with `a live value` at `dispatch.rs:1710`
    under stress.
  - **`substitution_array`'s root** (`messages.rs:250`). A probe whose `MAKEARRAY` answers a fresh
    `.array~of(...)` of objects prints `firstfirstfirstfirst /  / ` under stress.

At HEAD both probes match the normal run. Each needs a witness that fails when its root is
deleted.

### M2 (Minor): non-UTF-8 working directories

Task 6's `cd` resolves by bytes (`command.rs:313-372`), so it now enters a non-UTF-8 directory
exactly. The `Interp::cwd_text` lossy sites were unreachable through `cd` before 9b and are now
reachable. Probe: `'cd' 'dir'||'ff'x`, oracle beside it.

| what | crate | oracle |
|---|---|---|
| `DIRECTORY()` | ends `EFBFBD` | ends `FF` |
| `address system 'pwd'` | rc 127 | rc 0 |
| `SysCls()` | -1 | 256 |

- **`SysCls` is a new 9b site** (`builtin/rexxutil/process.rs:65`,
  `command.current_dir(interp.cwd_text())`), and the queued file
  `.superpowers/sdd/queued/2026-10-13-lossy-path-sites.md` does not list it.
- **The RexxUtil file routines handle such names exactly.** `SysFileCopy`, `SysFileTree` (by an
  absolute name and by `*` after the `cd`), `SysSearchPath`, `SysTempFileName` and `SysFileSearch`
  on `'f'||'fe'x||'.txt'` all match the oracle.

### M3 (Minor): a `~` with `HOME` unset has no oracle-crashes entry

Task 4 routed `~` through `qualify_name`, so it reaches `QUALIFY`, streams, `DIRECTORY`, every
RexxUtil path argument and `SysFileTree` (`expand_tilde`, `builtin/platform.rs:67`).

- **What the source shows on the oracle side:** `SysFileSystem::resolveTilde` does
  `name = getenv("HOME")` (`platform/unix/SysFileSystem.cpp:486`), and `FileNameBuffer::operator=`
  runs `strlen` on it (`memory/FileNameBuffer.hpp:122`). With `HOME` unset that is `strlen(NULL)`,
  which entry 44 measured killing the oracle's `cd ~`.
- **What the crate answers:** `''` for `~` and `/x` for `~/x`.
- **Not run on the oracle**, as the brief requires. The claim comes from the source.
- **Wanted:** an entry measured by whoever owns oracle runs, and a crate-only pin.

### M4 (Minor): oracle-crashes records

- **Entry 37 names no crate-only pin.** The pin exists: `stems/tests.rs` reads
  `tests/stem_sort_untrapped_cases/crate`.
- **Part of entry 45 has no pin.** Its sentence "a command whose child was reaped elsewhere answers
  `127` with a `FAILURE`" has no test, because `tests/rexxutil_process.rs` pins only `-1`. The
  behaviour itself is true: a probe printed `wait 1280`, `failure 127`, `cmd rc 127` in 3 runs of 3.
- Entries 38 to 44 each name a pin, and the named test exists.

### M5 (Minor): a "Phase 9" label with no Phase 9 owner

`Loud::fork_child` and `Loud::named_semaphore` (`lib.rs:507-535`) say `Some(Owner::Phase9)`.

- Moritz deferred SysFork "to 9b1 or later" and named semaphores "defer, keep refusal".
- Neither queued design names an owning sub-phase.
- At 9d's close, spec section 9 adds `"Phase 9"` to `closed_phases`' `CLOSED`, and these two rows
  will block that.
- The rulings themselves stand. The label states a future no plan holds. The owner needs a decision
  before 9d plans.

### M6 (Minor): prose

- Structuring semicolons:
  - `environment.rs:620`
  - `install.rs:633`
  - `tests/stream_handle.rs:68`, `:105`, `:131`
- ` -- ` at `crates/rexx-exec/Cargo.toml:67`, on a line 9b re-wrapped.
- A false sentence on a path 9b reworked: `lib.rs:2742` says `set_cwd`'s only caller is
  `DIRECTORY(new)`, but `cd` calls it at `command.rs:345`. This was already false before 9b.

### N1 (Note): the tz-rs dependency

The plan says "No new dependency" twice (plan lines 17 and 36), and Moritz approved the plan. The
Task 4 ruling added `tz-rs` with the reason "a crate is fine, builds are online". That reason is true
but answers whether the crate can be fetched, not the constraint itself. I am not re-litigating it.
I am raising it because the plan Moritz approved says otherwise.

The Cargo.toml comment's claims check out: `tz-rs-0.7.3/src/lib.rs:1` is `#![forbid(unsafe_code)]`,
and its Cargo.toml has no `[dependencies]` section.

## Checked and clean

- **No `unsafe` added.** `git diff -U0 f9105f68f..43a39ab01 -- rust | grep -E '^\+.*\bunsafe\b'`
  finds only prose, and `rexx-core/tests/unsafe_sites.rs` is unchanged.
- **Every spec section 4 routine has a `util_run` row** in `internal_routines.rs:110-235`.
  - `SysFork`, `SysOpenEventSem` and `SysOpenMutexSem` stay loud refusals that cite their queued
    designs.
  - The only `Phase10` rows in RexxUtil are the seven macrospace routines, each
    `Deferral::Rxapi`.
  - `handle_set` is implemented. `classic_style_routine` is a GUARD that records its search.
- **`cd` agrees with every other path that uses the working directory.** After `'cd lnk'` (a link
  to `real`), these match the oracle byte for byte, as do `cd ..` back out and `cd lnk/../sub`:
  - `DIRECTORY()` and `QUALIFY`
  - `LINEIN` on a relative name
  - `.File~exists`/`absolutePath`
  - `SysFileTree`, `SysSearchPath`
  - an external routine found in the new directory
- **Stream names with an empty string or a NUL** behave as on the oracle through `LINEOUT`,
  `CHAROUT`, `STREAM`, `LINES`, `CHARS` and `SysFileExists`.
- **The sort's `values` hold does not show** when deleted: the reqstr latch also roots those values.
  That is consistent with the Task 3 ruling that kept the hold explicit.
- **No new comment describes a mechanism that does not exist yet.** I grepped the added comment
  lines for will, until, later, not yet, eventually, 9b1, future and when built.
- **`collect_stress` at the archived HEAD:** 37 passed.

## Declined to judge

- **`a_move_across_file_systems_copies_and_unlinks` fails in the `/tmp` copy**, because the
  target directory and `/tmp` are one file system there. That is environmental: the Task 4 ruling
  made the test fail loudly when it cannot run.
- **The order of `SysDumpVariables`' lines differs from the oracle's** in my probe. Hash order is
  unspecified, so I did not compare it.
- **Performance, and gate exit statuses at the final HEAD.** The controller's re-run owns these.
- **`rexx_routine_objects` keyed by `ObjRef`.** This came before 9b, and 9b1 owns it.
