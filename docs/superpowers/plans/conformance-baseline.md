# Conformance baseline — the C++ oracle, Linux

What the ooTest suite reports against the **C++ interpreter** on this machine. This
is the target the Rust implementation must eventually match: "L3 green" means
reproducing these numbers, not producing zero failures.

Run 2026-07-27 against `plan/rust-rewrite` at `af593325`, built Release.

## Result

```
Interpreter:        REXX-ooRexx_5.3.0(MT)_64-bit 6.06 27 Jul 2026
OS Name:            LINUX
SysVersion:         Linux 7.1.3+deb14-amd64

Tests ran:          24372
Assertions:         391542
Failures:           2
Errors:             3

Test execution:     00:05:28.108636
Total time:         00:05:30.176061
```

Suite: 409 `.testGroup` files, 14,122 `::method test*` definitions, checked out from
`https://svn.code.sf.net/p/oorexx/code-0/test/trunk`. The 24,372 figure exceeds 14,122
because groups parameterise tests at run time.

## How it was run

```sh
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release -DCMAKE_POLICY_VERSION_MINIMUM=3.5
cmake --build build --parallel "$(getconf _NPROCESSORS_ONLN)"
svn checkout https://svn.code.sf.net/p/oorexx/code-0/test/trunk ootest
cd ootest
PATH="$PWD/../build/bin:$PATH" \
LD_LIBRARY_PATH="$PWD/../build/lib:$PWD/../build/bin" \
    rexx testOORexx.rex -s < /dev/null
```

**`< /dev/null` is required.** Without it the run hangs — not fails — at
`CHARIN.testGroup`, because `ADDRESS` and the CHARIN/CHAROUT groups start child
processes that read stdin. Commit `5ea8bc6c` fixed this for the BSD CI legs; it
applies to any local run too.

## The 2 failures — both expected

| Test | Status |
|---|---|
| `CHAROUT.testGroup/TEST_STDOUT_START` | listed in `known-test-failures/common.txt` |
| `LINEOUT.testGroup/TEST_LINE_TRANSIENT` | listed in `known-test-failures/common.txt` |

Both assert console behaviour and cannot pass when output is redirected, which `-s`
plus a shell redirect guarantees. The two `SYSSLEEP` entries in `common.txt` did not
fire on this run; that file permits a failure rather than requiring one.

## The 3 errors — none listed, and `linux.txt` is empty on purpose

`.github/known-test-failures/linux.txt` says it is empty deliberately, and that
"whatever the first Linux run reports gets investigated and then either listed here
with its reason or treated as a bug." These are that data.

### 1. `SysUnix.testGroup/TEST_SYSCRYPT` — environmental, not an interpreter bug

```
SYNTAX 43.1 raised unexpectedly.
Could not find routine "SYSCRYPT".   (line 360)
```

`SysCrypt` is compiled only under `#ifdef HAVE_CRYPT`
(`extensions/platform/unix/rxunixsys/rxunixsys.cpp:1515`, and the routine-table entry
at `:1717–1719`). This build has `/* #undef HAVE_CRYPT */` in `build/config.h`
because `libxcrypt-dev` is not installed — there is no `/usr/include/crypt.h`, and
`nm -D build/lib/librxunixsys.so` shows no crypt symbol. The routine genuinely does
not exist, so the test is right to fail; the *test* is at fault for not skipping when
the routine is absent.

**Consequence for CI:** the hosted Ubuntu image probably does ship libxcrypt, so this
error likely will not reproduce there. That makes this local run **not a substitute
for a CI run** when populating `linux.txt`.

### 2. `Array.testGroup/TEST_SORTWITH_BUG1466` — needs investigation

```
SYNTAX 4.1 raised unexpectedly.
Program interrupted with HALT condition.   (line 1818)
```

A HALT condition arriving unbidden during a sort regression test. Nothing sent an
interrupt. This one looks like a real defect and is worth chasing separately —
note that this repository has prior history with sort-related crashes.

### 3. `DateTime.testGroup/TEST_BRUTE_FORCE` — needs investigation

```
SYNTAX 88.918 raised unexpectedly.
Argument date is not in a valid format; found "0396-267".
```

An ordinal date (year 396, day 267) rejected as malformed. Either a genuine bug in
low-year ordinal parsing or a test generating a date outside the supported range.

## What this means for the rewrite

- The L3 gate is **proven runnable locally**, not merely specified. That was previously
  untested.
- The gate is "match the oracle", and the oracle is not clean. A Rust build reporting
  these same 5 results is passing; one reporting 0 is suspicious, not better.
- Errors 2 and 3 are pre-existing C++ defects or test defects. They are **not** the
  Rust project's to fix, but the Rust implementation must not be judged against a
  baseline that pretends they do not exist.
- Populating `linux.txt` needs a CI run, not this one, for the reason under error 1.

Nothing in this file has been added to `known-test-failures/`. That is a change to the
project's CI pass criteria and belongs to whoever owns that decision.
