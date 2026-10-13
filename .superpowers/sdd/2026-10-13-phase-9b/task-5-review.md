# Phase 9b Task 5 review: SysFileTree

Reviewer: t5 task reviewer, 2026-10-13. Range `c11344b31..cc6e05caa` (code in `813f71787`).
Crate binary: `rust/target/release/rexx-run` rebuilt at `cc6e05caa` (`Compiling rexx-exec` seen).
Every interpreter run from a fresh empty directory under the scratchpad, crate under `memcap 4G`
(or `2G` where stated) and `timeout`, oracle under the standard wrapper.

## Spec Compliance

✅ Spec compliant. The 17 tests the brief names are fixed by following `TreeFinder` and not by
shaping to the tests; every Review Focus 2 shape that applies is pinned, against the oracle where
the oracle has a defined answer.

⚠️ Cannot verify from the diff: that no derived row names this group. The report's
`grep -rn SysFileTree corpus/ crates/rexx-exec/tests/` is the right enumeration. I did not re-run
it.

## Strengths

- `tree.rs` follows the C++ step by step: argument marshalling, then `StemHandler` (40.925), then
  `validateFileSpec`/`getOptions`/`parseMask` in the constructor's order. It has the
  `adjustDirectory`/`getFullPath`/`fixupFilePath` split, the two-pass `recursiveFindFile`, and
  `formatFileAttributes` down to the hour rule and `(tm_year + 10000) % 100`. I checked each one
  against `RexxUtilCommon.cpp:311-740`, `SysRexxUtil.cpp:319-560`, `SysFileSystem.cpp:1870-2072`,
  and `RexxUtilCommon.hpp:305` (`AttributeMask::parseMask`).
- The corpus witness is broad and reads correctly. I re-ran it on both sides: stdout and stderr
  are byte-identical, rc 216 on both, and both run directories are empty afterwards.
- The validation-failure path writes tail `0` (`tree.rs` `file_tree`, `found.complete(interp, 0)`),
  matching `~TreeFinder` → `stemArray.complete()`.
- The refusal-sites re-derivation is as claimed. With the location column (3) cut, the only
  change is the new `routine_null_argument` row.

## Checks run

1. **Faithfulness of the 17 fixes.** I read each failing test's assertion
   (`ootest/.../SysFileTree.testGroup:164-504`) against the C++. The arg-error tests map to
   `nullStringException`, `badSFTOptsException` and `badMaskException`, with the same 256-byte
   `snprintf` cut (`OPTION_MESSAGE_LIMIT`). TEST_HUGE maps to `stat` (not `lstat`), the cap of
   `9999999999` and the widths `%10jd`/`%20jd`. The TREE tests map to F/D/B last-wins
   (`goodOpts`), the `.`/`..`/trailing-slash forms (`adjustDirectory`, `expandNonPath2fullPath`)
   and `I` (FNM_CASEFOLD). Nothing in `tree.rs` is keyed to a test's names or paths.
2. **Review Focus 2.** Witness lines, oracle answer = crate answer:
   - blank: `[b b|f]` `999  -rw-rw-r--  filetree.d/b b`
   - `'ff'x` beside the `'efbfbd'x` decoy: `[n*|o]` lists both, and `[n'ff'x|o]` lists exactly
     one
   - trailing slash: `[sub/|o]`, `[sub//|o]`, `[f/|o]` count 0
   - link to a directory: `[link|]` `drwxrwxr-x`, `[link|f]` count 0, `[link/|o]` and `[*|so]`
     recurse into `link/F`
   - unreadable directory: `[closed|]` `d---------`, `[closed/*|so]` count 0
   - dangling link: crate-only (`tree/tests.rs`), per the ruling.
3. **Symlink loop** (`loop.d/s/up -> ..`, `SysFileTree('loop.d/*', f., 's')`). Both sides
   terminate at the kernel's 40-link limit with count 123, and the crate's peak RSS is 20 MB.
   The deepest `up` entry (41 links, so `stat` fails with ELOOP) differs. The oracle prints
   its stale buffer (`80  drwxrwxr-x`, the previous entry's data). The crate prints
   `2  lrwxrwxrwx`, from `lstat`. Same mechanism as entry 40, so it falls under the dangling-link
   ruling. A tree whose loops fan out (`/usr/share` here has 233 loops under `find -L`) grows
   exponentially on both sides: the crate was OOM-killed at `memcap 2G`, and the oracle had not
   finished at 120 s. That is not a divergence.
4. **TEST_HUGE under memcap 2G.** `SysFileTree('/dev/core', f., 'tf')` and `'tfh'` print
   `9999999999` and `140737471594496`, identical to the oracle. Peak RSS is 20.05 MB, against
   19.39 MB for `say 1` (`/usr/bin/time` as the direct parent of `rexx-run`, inside `memcap`).
   Nothing is allocated in proportion to the file. Allocation is proportional to the result: a
   loop-free `/usr/include/*` `sb` (7529 lines, 652515 bytes) peaks at 23.1 MB, and its count
   equals the oracle's.
5. **GC rooting.** The search builds `Vec<Vec<u8>>`, so no heap object exists until the
   results loop. `Found::add` then makes one `text_built` and writes it before any further
   `alloc_with`: `stem_object_write` allocates no object, and `append_slot`'s `array_grow` only
   calls `charge_growth`. The target is rooted as an argument or by its variable. `complete`'s
   `answer` allocates after the last write. Sound.
6. **Perf** (`bench-programs/callgrind.sh -r 3 -j 4 -p "dirread startup" -B 0.5
   pin=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run head=<copy of HEAD binary>`, exit 0):

   | program | pin | head | whole-process Δ | −libc Δ | spread |
   |---|---|---|---|---|---|
   | dirread | 3130113973 | 3143810221 | +0.4376% | +0.4987% | 0.0003% |
   | startup | 119095681 | 119430558 | +0.2812% | +0.3693% | 0.0035% |

   Within +0.5%. dirread was +0.39% after Task 4, so the pin-relative headroom left for 9b is
   about 0.06%.
7. **Mutations.** I built them in a `git archive` copy with its own target dir, so the checkout
   was not touched. The copy's target was deleted afterwards. Both mutants turn the witness red:
   - `stat` → `lstat` in `entries`: 28 diff lines (link `l` and counted under F, no `link/F`
     under `S`)
   - cap removed: four `big` lines print `21474836480`

## Issues

### Critical

None.

### Important

1. **`matches` lacks POSIX equivalence classes and collating symbols**
   (`rust/crates/rexx-exec/src/builtin/rexxutil/tree.rs` `bracket`). The doc of `file_tree` says
   names are "matched as `fnmatch(3)` matches them". glibc in the C locale accepts `[=c=]` and
   `[.c.]`, and the crate treats them as literal bytes. Measured on a directory holding
   `a b - ] ^ ! x.y A`:

   | pattern | oracle | crate |
   |---|---|---|
   | `[[=a=]]` | `a` | nothing |
   | `[[.a.]]` | `a` | nothing |
   | `[[.-.]]` | `-` | nothing |
   | `[a-[.c.]]` | `a b`, and `A a b` with `I` | nothing |
   | `[[=a=]b]` | `a b` | nothing |

   These bracket forms agree (`[a-]`, `[]-a]`, `[!]]`, `[[:alpha:][:digit:]]`, `[\]]`,
   `[[:ALPHA:]]`, `[[:alpha:]`, `[[]`, `*[`, `[!-]`, `[a-c-e]`, each with and without `I`). The
   fix: in the C locale, `[=c=]` and `[.c.]` with a single byte `c` are that byte, also as a
   range end. Add the rows to `bracket_expressions_match_as_fnmatch_does`, or as witness lines.
   The report's "every bracket form in the unit test was checked" was true but did not cover
   these.
2. **The `normalize_bytes` repro is confirmed, and the defect is wider than a doubled slash.**
   This is input for the ruled fix. `say qualify(n)`, oracle then crate:

   | input | oracle | crate |
   |---|---|---|
   | `/a/b//../c` | `/a/b/c` | `/a/c` |
   | `/a/b///../c` | `/a/b/c` | `/a/c` |
   | `/a/b/.//../c` | `/a/b/c` | `/a/c` |
   | `/a/b//..` | `/a/b` | `/a` |
   | `/a/b/c/..//../d` | `/a/b/d` | `/a/d` |
   | `/a/b/c//../../d` | `/a/b/d` | `/a/d` |
   | `/a/.hidden/../c` | `/c` | `/a/c` |
   | `/a/b/..x/../c` | `/a/c` | `/a/b/c` |
   | `/a/b/.../../c` | `/a/c` | `/a/b/c` |

   Agreeing controls: `/a/.b/../../c`, `/a/b/./../c`, `/a//b/../c`, `//a/../b`, `/a/./../b`,
   `/a/b/../../..`.

   **The rule, from `SysFileSystem::normalizePathName`** (`platform/unix/SysFileSystem.cpp:687`).
   It is a byte loop over an output buffer with indices `dest` and `previousSlash`, and
   `res[0] = '/'`.
   - At each input `/`: if the next input byte is not `.`, set `previousSlash = dest` *before*
     deduplicating. Then skip the `/` if `res[dest] == '/'`, else append it.
   - At `.` with `res[dest] == '/'`:
     - if the next byte is NUL or `/`, skip it;
     - if it is `..` followed by NUL or `/`, consume both, set `dest = previousSlash`, then walk
       `previousSlash` back to the previous `/` in `res` (`while previousSlash > 0 { if
       res[--previousSlash] == '/' break }`);
     - otherwise append it.
   - Every other byte is appended.
   - At the end, a trailing `/` at `dest != 0` is dropped.

   Two consequences follow, and both produce the table above:
   - A `/` followed by another `/` records the position *before* it, so `..` right after a
     doubled slash removes nothing.
   - A component that starts with `.` (`.hidden`, `..x`, `...`) never updates `previousSlash`, so
     a later `..` pops back past it *and* its parent.

   The fix ports the loop verbatim, and the QUALIFY witness carries every row above. A fix that
   handles only the doubled slash leaves the last three rows diverging. **Ruling wanted:** the
   dot-component behaviour is an upstream bug that makes `qualify('/a/.config/../x')` answer
   `/x`. Matching it is the project's default, but it is user-visible and unfiled.

### Minor

1. `entries` (`tree.rs`) `stat`s every entry in the matching pass. `findNextEntry` stats only
   the names `fnmatch` accepted. Output is the same, but the crate does more syscalls and can
   touch entries (for example autofs triggers) that the oracle never stats. Filter by `matches`
   before `stat` in the first pass.
2. The ELOOP entry (check 3) shows a second shape of the entry-40 mechanism. Entry 40's text and
   the doc on `a_dangling_link_is_listed_by_its_own_status` could say "any `stat` failure
   (dangling, ELOOP)" so the next reader does not take the loop as unexplained.
3. `entries` drops an entry when `stat` and `lstat` both fail (a name removed between `readdir`
   and `stat`). The oracle lists it with stale data. This is a race only and does not need a
   change, but the doc's "Where `stat` fails ... it is `lstat`'s answer" is silent on it.

## Assessment

**Task quality:** Needs fixes. Important 1 needs a small `bracket` change and new rows. Important
2 is already ruled into this fix round, and the rule above is what it should implement.

**Reasoning:** The port is faithful, witnessed on every Focus 2 shape, rooted correctly, bounded
in memory, within budget, and its witness catches mutants. The one new divergence is an
unsupported bracket form in a matcher the doc says behaves as `fnmatch`.

## Probe programs

QUALIFY (`q.rex`):

    list = '/a/b//../c /a/b///../c /a/b/.//../c /a/b//.. /a/.hidden/../c /a/.b/../../c /a/b/./../c /a/b/c/..//../d /a//b/../c //a/../b /a/b/..x/../c /a/./../b /a/b/../../.. /a/b/.../../c /a/b/c//../../d'
    do i = 1 to words(list)
      n = word(list, i)
      say left(n, 22) qualify(n)
    end

Brackets (`g2.rex`, abridged): it makes `g.d` holding the files `a b - ] ^ ! x.y A`, runs
`SysFileTree('g.d/'p, f., 'o')` and `'oi'` for each pattern above, and prints the sorted base
names.

Loop (`loop.rex`): it makes `loop.d/s` and the file `loop.d/a`, runs `'ln -s .. loop.d/s/up'`,
then `SysFileTree('loop.d/*', h., 's')`, and prints the count and the entries with 40 or more
`/up`.
