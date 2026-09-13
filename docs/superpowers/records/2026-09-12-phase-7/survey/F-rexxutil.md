# Survey F — RexxUtil (`Sys*`) subset

Surveyor F, Phase 7. Area: the `RexxUtil` (`Sys*`) library subset Phase 7 must build, derived from
what the test framework and non-RexxUtil test groups actually call (D11), plus how RexxUtil is
resolved into the namespace and where it should refuse loudly for the Phase-10 remainder.
Probe root: `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/survey-F/pNN/`.
Oracle: C++ trunk build at `/home/moritz/dev/repos/ooRexx/build`. Crate: snapshot binary
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/bins/h-5bcb28edb`.

Claim marks: **Measured** (probe run, three descriptors quoted), **Read** (file:line), **Inferred**.

**Probe-hygiene note, recorded against myself.** One probe in this survey (`ft-p01/prog2.rex`, the
`SysFileTree` option sweep) was first run without `cd`-ing into its probe directory inside the same
Bash call that invoked the oracle — this environment resets the shell's cwd between tool calls, so
the redirects `>out2.txt 2>err2.txt` landed relative to the repository root instead of the probe
directory, and the program's own `SysFileTree "*"` search (with the `S` recursive option) walked
from the repository root rather than the probe directory. Caught immediately from the output size
(8.7 MB) and the missing files at the expected path; the three stray files (`out2.txt`, `err2.txt`,
`rc2.txt`) written into the repository root were deleted before anything else ran, and
`git status --porcelain` confirmed the tree was back to the session's starting state (only the
pre-existing `M rust/corpus/oracle-crashes.txt` and the untracked Phase-7 spec file, neither mine).
No file was written, deleted or renamed outside the probe directory as a result — the command that
went astray was a read-only directory listing, not a mutation. The probe was re-run correctly
immediately after, `cd`-ing into the probe directory within the same Bash invocation as the oracle
call, and its numbers below are from that re-run.

## 0. Deriving the subset

D11 (`docs/superpowers/plans/2026-07-27-rust-rewrite.md:299-332`) ordered the whole suite's `Sys*`
call sites by raw count across all 409 `.testGroup` files. That count mixes three things that need
to be told apart: real calls from the framework, real calls from *other* areas' test groups (the
signal Phase 7 needs), and call sites inside RexxUtil's own test groups, which test RexxUtil and
say nothing about who else needs it. This section re-derives the subset by the method the brief
asks for: (a) the framework, (b) non-RexxUtil test groups, ranked by distinct file count.

### (a) The framework

RexxUtil's own test groups live under `ootest/ooRexx/base/rexxutil/` (30 `.testGroup` files,
including a `platform/unix` and `platform/windows` split) — established by:
```
find ootest/ooRexx/base/rexxutil -name "*.testGroup" | wc -l    # 30
```
The framework files are `ootest/testOORexx.rex`, `ootest/worker.rex`, `ootest/ooTest.frm`, and
everything under `ootest/framework/`. Naive `grep -oE 'Sys[A-Za-z]+\('` misses the `CALL name args`
form (no parens), which the framework actually uses once, so the derivation used both forms:
```
/bin/grep -noE '[^A-Za-z0-9_]Sys[A-Za-z]+\(' ootest/testOORexx.rex ootest/worker.rex ootest/ooTest.frm \
  ootest/framework/runTestUnits.rex ootest/framework/WinUtils.cls ootest/framework/FileUtils.cls \
  ootest/framework/OOREXXUNIT.CLS ootest/framework/runXSLT.rxj 2>/dev/null | /bin/grep -oE 'Sys[A-Za-z]+' | sort | uniq -c
```
found real (paren) calls to `SysFileTree`, `SysFileExists`, `SysSleep`, `SysVersion`, `SysWinVer`,
`SysLinVer`. Re-reading each hit in context (`sed -n` around every line number) to throw out
comments and strings left:

| Name | Site | Form |
|---|---|---|
| `SysFileExists` | `worker.rex:318`, `:364` | `SysFileExists(self~DEFAULT_OPTIONS_FILE)` / `SysFileExists(optFile)` |
| `SysFileTree` | `ooTest.frm:2126`, `:2131` | `SysFileTree(self~file, files., "FOS")` |
| `SysFileTree` | `framework/runTestUnits.rex:56` | `call sysFileTree searchFile, "tests.", switches` — **`CALL` form, invisible to a paren-only grep**; the three paren-shaped hits the naive grep found at `:53,131,133` in this file are all inside a `say` string or a comment naming the option letters, not calls |
| `SysSleep` | `ooTest.frm:2455` | `j = SysSleep(.5)` |
| `SysSleep` | `framework/WinUtils.cls:123` | `j = SysSleep(pause)` |
| `SysVersion` | `ooTest.frm:721-723` | unconditional |
| `SysWinVer` | `ooTest.frm:724-726,730` | guarded: `if rxfuncquery("SysWinVer") = 0, ... then` |
| `SysLinVer` | `ooTest.frm:727-730` | guarded: `if rxfuncquery("SysLinVer") = 0, ... then` |

`SysWinVer` is excluded: it is Windows-only (§1's doc table) and its own call site checks
`rxfuncquery("SysWinVer") = 0` first — on Linux that function is not registered, `RXFUNCQUERY`
answers non-zero, and the guarded branch never runs. **Read**, not measured — the interpreter's own
comma-conjunction reading inside that `IF` was not independently re-derived here, only that
`SysWinVer` never appears anywhere else and is documented Windows-only, which is enough to drop it
regardless of exactly how the guard parses. **(a) is: `SysFileExists`, `SysFileTree`, `SysSleep`,
`SysVersion`, `SysLinVer`.**

### (b) Non-RexxUtil test groups, ranked by distinct file count

```
find ootest/ooRexx -name "*.testGroup" | wc -l                                        # 402
find ootest/ooRexx/base/rexxutil -name "*.testGroup" | wc -l                          # 30
find ootest/ooRexx -name "*.testGroup" -not -path "ootest/ooRexx/base/rexxutil/*" | wc -l   # 372
find ootest -name "*.testGroup" | wc -l                                               # 409 (matches D11)
```
The other 7 (`ootest/misc/*.testGroup`: `template`, `templateAPI`, `Simplest`, `Advanced`, …) are
framework example/template files under a directory literally named `misc`, not suite content the
default runner discovers — excluded from (b) rather than counted.

Candidate names, any `Sys[A-Za-z]+` word anywhere in the 372 non-RexxUtil files:
```
/bin/grep -ohE '\<Sys[A-Za-z]+\>' $(cat nonrexxutil.lst) 2>/dev/null | sort -u    # 47 candidates
```
Ranked by **distinct files** containing a real call — `Name(` or `call Name` — not raw occurrence
count:
```
for name in <candidate>; do
  n=$(/bin/grep -liE "(^|[^A-Za-z0-9_])${name}\(|(^|[^A-Za-z0-9_])call[ \t]+${name}([^A-Za-z0-9_]|$)" $(cat nonrexxutil.lst) | wc -l)
  echo "$n $name"
done | sort -rn
```
Result: `SysSleep` 23, `SysFileDelete` 10, `SysFileExists` 7 (already in (a)), `SysIsFile` 3,
`SysSystemDirectory` 2, `SysRmDir` 2, `SysMkDir` 2, then 33 names at exactly 1, then 5 at 0 (grep
noise: `System`, `SystemRoot`, `SystemCode`, `SysFile`, `SysUnix` are substrings/false positives,
never called).

**A whole cluster of the singletons is not RexxUtil at all.** `SysGetuid`, `SysGetpid`,
`SysGetegid`, `SysGeteuid`, `SysGetgid`, `SysGetgrgid`, `SysGetgrnam`, `SysGetpwnam`,
`SysGetpwuid`, `SysGetservbyname`, `SysGetservbyport`, `SysGetsid`, `SysGettid`, `SysGetppid`,
`SysGetXattr`, `SysListXattr`, `SysRemoveXattr`, `SysSetXattr`, `SysStat`, `SysUname`,
`SysWordExp`, `SysCrypt`, `SysAccess` all occur in exactly one file,
`ootest/ooRexx/extensions/platform/unix/rxunixsys/SysUnix.testGroup`, whose own prolog is
`.context~package~loadLibrary("rxunixsys")` (line 43) — a **native shared library** loaded by
`::REQUIRES ... LIBRARY`-equivalent means, which the brief's own scope ruling already sends to
**Phase 8**, not RexxUtil. None of these 22 names are in RexxUtil's registration table either (§3).
**Measured/Read**: `/bin/grep -n "loadLibrary" ootest/ooRexx/extensions/platform/unix/rxunixsys/SysUnix.testGroup` → line 43.

The remaining singletons are each explained away individually: `SysBootDrive` (`File.testGroup:946`)
only appears inside `call SysFileTree SysBootDrive() || "\" || system, ...` checking for
`hiberfil.sys`/`swapfile.sys`/`pagefile.sys` — Windows artifacts, and `SysBootDrive` itself is
documented Windows-only. `SysWinGetPrinters` and `SysToUnicode` are each in a
`platform/windows/...testGroup` file. `SysDropFuncs` is not documented in `rexxutil.xml` at all
(confirmed: `/bin/grep -n SysDropFuncs oodocs/rexxref/en-US/rexxutil.xml` → no hit) though it **is**
a real registered routine (§3) — the security-manager test's own comment says why it is there:
*"SysDropFuncs() is a no-op; just good for testing"* (`SecurityManager.testGroup:132`) — used purely
as a stand-in external-call name to exercise the security manager's `CALL` trigger, never for its
own behaviour. `SysSearchPath` (`SecurityManager.testGroup:143`) is the same shape — a real,
documented RexxUtil function, but the one call site is `return SysSearchPath('foo', 'bar', 'c')`
inside a security-manager mock, exercising the trigger mechanism rather than the function.
`SysSystemDirectory`'s two hits are both platform-gated: `platform/windows/ole/SpecialFolders.testGroup`
is a Windows-only file, and its other site (`File.testGroup:850`) is
`if .RexxInfo~platform~caselessStartsWith("WINDOWS") then root = SysSystemDirectory() else root = "/"`
— never taken on Linux.

**Threshold: distinct-file count ≥ 2, after removing the rxunixsys cluster, Windows-gated call
sites, and the two placeholder-only names.** That keeps `SysFileDelete`, `SysIsFile`, `SysRmDir`,
`SysMkDir` from (b) (`SysFileExists`/`SysSleep` already in (a)); it drops `SysSystemDirectory`
despite its raw count of 2, because both its files are Windows-only in effect, and it drops
`SysSearchPath`/`SysDropFuncs` despite being real/registered, because neither test exercises the
function itself. A lower threshold would only add rxunixsys names, Windows-only names, or more
placeholder call sites — every single-file hit was one of those three shapes once traced, which is
itself a finding: the culling step, not the threshold number, is doing the real work here.

**Proposed subset (9 functions): `SysFileExists`, `SysFileTree`, `SysSleep`, `SysVersion`,
`SysLinVer`, `SysFileDelete`, `SysIsFile`, `SysRmDir`, `SysMkDir`.**

## 1. Documented surface

Every `<section id="utlSys...">` in `oodocs/rexxref/en-US/rexxutil.xml`, 72 total —
`/bin/grep -c '<section id="utlSys' oodocs/rexxref/en-US/rexxutil.xml`. The doc marks platform
scope **in the section title itself** (`(Windows only)`, `(Unix-like systems only)`,
`(Linux Only)`, or no marker for cross-platform) — that marker, not a guess, is what "Windows-only"
means below.

Cross-platform, non-macrospace (29) — candidates for a Phase 7/10 RexxUtil implementation:

| Item | Doc | Contract |
|---|---|---|
| `SysCls` | `:707` | Clears the screen (ANSI). |
| `SysDumpVariables` | `:940` | Writes all variables in the current scope to a file. |
| `SysFileCopy` | `:991` | Copies a file. |
| `SysFileDelete` | `:1039` | Deletes a file; return code 0 or an errno-shaped "other". **In subset.** |
| `SysFileExists` | `:1108` | 1 if a file or directory exists, else 0. **In subset.** |
| `SysFileMove` | `:1147` | Moves/renames a file. |
| `SysFileSearch` | `:1195` | Searches a file's lines for a substring into a stem. |
| `SysFileTree` | `:1343` | Lists files/dirs matching a glob into a stem, with attribute/date options. **In subset.** |
| `SysFormatMessage` | `:1636` (added 5.0) | `printf`-style substitution into a message string. |
| `SysGetErrorText` | `:1891` | Maps a numeric error code to platform error text. |
| `SysGetFileDateTime` | `:1935` | Returns a file's date/time. |
| `SysGetKey` | `:2012` | Reads one key from the console, no echo. |
| `SysIsFile` | `:2397` | 1 only for a regular file (block devices count on Unix); 0 for a directory. **In subset.** |
| `SysIsFileDirectory` | `:2472` | 1 if the name is a directory. |
| `SysIsFileLink` | `:2545` | 1 if the name is a symbolic link. |
| `SysMkDir` | `:2788` | Creates one directory (no intermediate dirs); Unix-only `mode` parameter. **In subset.** |
| `SysQueryProcess` | `:2875` | Returns process id / parent id / process type. |
| `SysRmDir` | `:3001` | Removes an empty directory. **In subset.** |
| `SysSearchPath` | `:3102` | Searches a `PATH`-like variable's directories for a file. |
| `SysSetFileDateTime` | `:3166` | Sets a file's date/time. |
| `SysSetPriority` | `:3235` | Sets process/thread scheduling priority/class. |
| `SysSleep` | `:3405` | Pauses the program for a number of seconds. **In subset.** |
| `SysStemCopy` | `:3442` | Copies a range of a stem to another stem. |
| `SysStemDelete` | `:3523` | Deletes a range of stem entries, shifting the rest down. |
| `SysStemInsert` | `:3579` | Inserts entries into a stem at a position. |
| `SysStemSort` | `:3624` | Sorts a stem's entries in place. |
| `SysTempFileName` | `:3746` (changed 5.0) | Generates a unique temporary file name from a template. |
| `SysUtilVersion` | `:4145` | Returns the RexxUtil library's own version string. |
| `SysVersion` | `:4163` | Returns `"<OS name> <version>"`. **In subset.** |

Macrospace, cross-platform but explicitly Phase 10/RXAPI per the brief (7, not further surveyed
here): `SysAddRexxMacro` (`:650`), `SysClearRexxMacroSpace` (`:692`), `SysDropRexxMacro` (`:919`),
`SysLoadRexxMacroSpace` (`:2765`), `SysQueryRexxMacro` (`:2954`), `SysReorderRexxMacro` (`:2976`),
`SysSaveRexxMacroSpace` (`:3080`).

Unix-like systems only (5): `SysCreatePipe` (`:726`) — opens a pipe, returns two stream tokens;
`SysFork` (`:1557`) — `fork(2)`, returns 0/pid/negative; `SysGetMessage` (`:2084`) and
`SysGetMessageX` (`:2147`) — looks up a message-catalog string, with/without substitutions;
`SysWait` (`:4226`) — waits for a child process. None reached the (a)/(b) threshold above (checked:
none of these five names appears in the (a) or (b) grep output at all), so none are in the proposed
subset, but they are cross-platform-relevant (Linux-supported) and squarely RexxUtil's, unlike the
Windows-only 30 below — flagged for Phase 10 rather than assumed absent.

Linux only (1): `SysLinVer` (`:2727`) — same contract as `SysVersion`, and (§3) the identical C
function under the `LINUX` build flag. **In subset.**

Windows only (30, established by the doc's own title marker, not by guessing): `SysBootDrive`,
`SysCurPos`, `SysCurState`, `SysDriveInfo`, `SysDriveMap`, `SysFileSystemType`, `SysFromUnicode`,
`SysGetLongPathName`, `SysGetShortPathName`, `SysIni`, `SysIsFileCompressed`, `SysIsFileEncrypted`,
`SysIsFileNotContentIndexed`, `SysIsFileOffline`, `SysIsFileSparse`, `SysIsFileTemporary`,
`SysShutdownSystem`, `SysSwitchSession`, `SysSystemDirectory`, `SysTextScreenRead`,
`SysTextScreenSize`, `SysToUnicode`, `SysVolumeLabel`, `SysWaitNamedPipe`, `SysWinDecryptFile`,
`SysWinEncryptFile`, `SysWinGetDefaultPrinter`, `SysWinGetPrinters`, `SysWinSetDefaultPrinter`,
`SysWinVer`. None are in the proposed subset; not surveyed further here (Phase 10, or never — this
crate has no Windows target today).

**Registered but wholly undocumented** (found in §3's registration table, not in `rexxutil.xml` at
all): `SysDropFuncs`, `SysLoadFuncs` (two more RXAPI-shaped legacy no-ops/wrappers — `SysDropFuncs`
is exercised only as the security-manager placeholder above; `SysLoadFuncs` was not seen called
anywhere in ootest), and eleven Unix semaphore functions
(`SysCreateMutexSem`/`SysOpenMutexSem`/`SysCloseMutexSem`/`SysRequestMutexSem`/`SysReleaseMutexSem`/
`SysCreateEventSem`/`SysOpenEventSem`/`SysCloseEventSem`/`SysResetEventSem`/`SysPostEventSem`/
`SysWaitEventSem`, all `#if !defined __APPLE__` in `interpreter/platform/unix/SysRexxUtilFunctions.h:3-15`).
None are called anywhere in ootest outside their absence; out of scope for the proposed subset and
not surveyed further — named here only because §3 asked for the full registration-table list.

## 2. Oracle behaviour

### 2.1 How RexxUtil is made available, and where it sits in resolution — measured

Three probes from a fresh directory (`ORACLE` is the standard wrapped invocation from the brief).

**A local `::ROUTINE` of the same name wins over RexxUtil's own**, for the function-call form:
```rexx
say SysFileExists('.')
::routine SysFileExists
  return "ROUTINE WINS"
```
stdout `ROUTINE WINS`, stderr empty, rc 0.

**A local internal label wins over RexxUtil's own**, for the `CALL` form (labels are not visible to
function-call syntax at all, so this had to be tested with `CALL`, not `name(...)`):
```rexx
call SysFileExists '.'
say result
exit
SysFileExists:
  return "LABEL WINS"
```
stdout `LABEL WINS`, stderr empty, rc 0.

**RexxUtil wins over an external file of the same name in the same directory** — i.e. it resolves
*before* the external-file search, not after:
```rexx
-- SysFileExists.rex, same directory:
say "EXTERNAL FILE WINS"
return 42
-- caller.rex:
say SysFileExists('.')
```
Running `caller.rex`: stdout `1`, stderr empty, rc 0 — RexxUtil's own answer, not the file's.

So the resolution order is: local label (`CALL` only) / local `::ROUTINE` (either form) →
**RexxUtil (and, on this evidence, the internal `REXX` package the same way — see §3)** → external
file. This matches survey D's finding for `DIRECTORY`/`FILESPEC`/`BEEP` exactly, and §3 gives the
single C++ mechanism (`PackageManager::initialize`, `resolveRoutine`) behind both.

### 2.2 The proposed subset, normal/edge/error cases

All from a fresh probe directory; `.` denotes the probe directory itself.

**`SysFileExists`** — `say SysFileExists('.')` → `1`; on a plain file → `1`; on a missing name →
`0`; on `''` → `0`. Directories count as "exists", matching the doc. Arity: `SysFileExists()` →
stderr `Error 43 ... Compiled routine "SYSFILEEXISTS". ... Error 88 ... Invalid argument. Error
88.901:  Missing argument; argument 1 is required.`, rc 168. `SysFileExists('a','b')` → same shape,
`Error 88.922:  Too many arguments in invocation; 1 expected.`, rc 168.

**`SysIsFile`** — on a directory → `0` (differs from `SysFileExists`); on a plain file → `1`; on a
missing name → `0`. Confirms the doc's distinction between the two functions.

**`SysFileDelete`** — on an existing plain file → `0`, and the file is gone afterward (`ls`
confirmed). On a missing name → **`13`**, not the `2` (`ENOENT`) one would expect — see §3, this is
a real oracle quirk from a pre-check, not the raw delete errno. On an existing directory → `21`
(`EISDIR`, raw errno from the failed `unlink(2)`).

**`SysRmDir`** — on an empty directory → `0`, directory gone afterward. On a non-empty directory →
`39` (`ENOTEMPTY`, raw errno). On a missing name → `2` (`ENOENT`, raw errno — this one *is* the
literal `remove(2)` errno, unlike `SysFileDelete`).

**`SysMkDir`** — on a new name → `0`, directory created (`ls -ld` confirmed, default mode
`rwxrwxrwx` before umask). On an existing name → `17` (`EEXIST`, raw errno). With an explicit
`mode` argument (`SysMkDir('modeddir', 448)`, i.e. octal 700) → `0`, and `ls -ld` showed
`drwx------`, confirming the Unix-only second argument is a raw permission bits value applied
verbatim (`448` decimal = `0700` octal, matching the doc's "0 (octal 000) to 511 (octal 777)").

**`SysSleep`** — `SysSleep(0)` → `0`; `SysSleep(0.05)` → `0` (fractional seconds accepted).
`SysSleep(-1)` → stderr `Error 88 ... Compiled routine "SYSSLEEP". ... Invalid argument. Error
88.907:  Argument delay must be in the range 0 to 2147483; found "-1"`, rc 168. **The doc
(`:3423`) claims the Unix-like upper bound is 999999999; the oracle's own error text says
`2147483` on this Linux build** — see §3, the C++ source enforces the Windows-derived bound
unconditionally, so the doc is stale for Unix, not the implementation. Not independently
re-measured at the boundary itself (sleeping 2147483 seconds is not a probe this survey can afford)
— the range comes from the C++ source (§3), corroborated by the error message's own numbers.

**`SysVersion`** / **`SysLinVer`** — both `say SysVersion()` and `say SysLinVer()` on this machine
print `Linux 7.1.12+deb14-amd64` (matches `uname -sr`, confirmed identical to what `uname` would
report locally). Identical output is expected, not coincidental: §3 shows `SysLinVer` is registered
as a second name for the exact same C function as `SysVersion`, guarded by `#ifdef LINUX`. **Host-
dependent**: this string will differ on any other kernel/build; the source of truth is `uname(2)`'s
`sysname` and `release` fields (§3), not a fixed value.

**`SysFileTree`**, from a directory containing `afile.txt`, `bfile.txt`, `sub1/nested.txt` (plus the
probe's own `prog.rex`/`out.txt`/`err.txt`):
- Default (`"B"`, both files and dirs, no `O`): one stem line per entry, exact bytes
  ` 9/12/26   2:47a           0  -rw-rw-r--  <fully-qualified-path>` for a file and
  ` 9/12/26   2:47a          60  drwxrwxr-x  <fully-qualified-path>` for a directory — i.e. **the
  path is already fully-qualified even without `O`**; `O` instead removes the date/size/attribute
  columns entirely, leaving bare `<fully-qualified-path>` per line (confirmed: `"FO"` gave
  9 one-column lines, one per file in the flat directory). Order was `err.txt, out.txt, prog.rex,
  bfile.txt, afile.txt` for the plain-file run and similar in the second — **not alphabetical**,
  matching the doc's "order they are found by the operating system, no specific order can be
  assumed" and directly corroborating survey B's readdir-order finding for `.File`.
- `"DO"` (directories only, name-only): exactly the one subdirectory, `sub1`.
- `"FOS"` (files, name-only, recursive): includes `sub1/nested.txt` at the end, after the
  top-level files — confirms recursion descends after finishing the current level.
- `"FL"` (long date): ` 2026-09-12 02:47:36           0  -rw-rw-r--  <path>` — exact
  `YYYY-MM-DD HH:MM:SS` as documented.
- `"FT"` (combined date): `26/09/12/02/47           0  -rw-rw-r--  <path>` — exact
  `YY/MM/DD/HH/MM` as documented, two-digit year, no seconds.
- No match (`"nomatch*.zzz"`): `result` (the function's own return code) is `0` and the stem's
  `.0` is `0` — **zero matches is success, not an error.**
- `"I"` case-insensitive, against a file named `UPPER.TXT`, searching for `"upper.txt"`: without
  `I`, 0 matches (the underlying filesystem is case-sensitive); with `I`, 1 match, and the returned
  name preserves the file's actual on-disk case (`UPPER.TXT`), not the search pattern's case.
- Arity: `call SysFileTree "*"` (stem argument omitted) → `Error 88.901:  Missing argument;
  argument 2 is required.`, rc 168 — `filespec` and `stemarray` are both required, matching the
  doc.

### 2.3 The crate's refusal today — measured across the subset plus DIRECTORY/FILESPEC/BEEP

One-line probe per name, `say NAME()`, run against the crate snapshot binary. All twelve sampled —
the 9-function subset plus the three internal-package routines survey D covers — answer identically:
```
Error 43 running <path> line 1:  Routine not found.
Error 43.1:  Could not find routine "<NAME-UPPERCASED>".
```
rc 213 in every case, stdout empty, for: `SysFileExists`, `SysFileTree`, `SysSleep`, `SysVersion`,
`SysLinVer`, `SysFileDelete`, `SysIsFile`, `SysRmDir`, `SysMkDir`, `DIRECTORY`, `FILESPEC`, `BEEP`.
This is a **trappable Rexx condition at a normal exit code**, not a loud refusal — confirming the
brief's framing exactly, and extending it from the one example (`SysFileExists`) to the whole
proposed subset plus the three names survey D already established.

## 3. C++ mechanism

### 3.1 Registration — the committed source for the full name list

`interpreter/package/PackageManager.cpp:88`, inside `PackageManager::initialize()`:
```cpp
loadInternalPackage(GlobalNames::REXX, rexxPackage);
loadInternalPackage(GlobalNames::REXXUTIL, rexxutilPackage);
```
Both the `REXX` package (survey D's `DIRECTORY`/`FILESPEC`/`BEEP`) and the `REXXUTIL` package are
**internal, always loaded, unconditionally, at interpreter startup** — this is not the OS/2-legacy
`RxFuncAdd` model where a program must register the library first; every process gets both, and
`PackageManager::restore()` (`:143-160`) reloads exactly these same two internal packages when
restoring from the saved image. `rexxutil_package_entry` (`RexxUtilCommon.cpp:2194-2202`) names the
package `"REXXUTIL"`.

The routine table is built in two pieces, both feeding the same C array
(`RexxUtilCommon.cpp:2158-2171`, closed by `#include "SysRexxUtilFunctions.h"` at `:2170` before
`REXX_LAST_ROUTINE()`):
- An explicit, platform-independent list in `RexxUtilCommon.cpp` itself: `SysFileTree`,
  `SysUtilVersion` (twice — harmless duplicate registration), `SysAddRexxMacro`,
  `SysDropRexxMacro`, `SysReorderRexxMacro`, `SysQueryRexxMacro`, `SysClearRexxMacroSpace`,
  `SysLoadRexxMacroSpace`, `SysSaveRexxMacroSpace`, `SysDropFuncs`, `SysLoadFuncs`,
  `SysDumpVariables`, `SysStemSort`, `SysStemDelete`, `SysStemInsert`, `SysStemCopy`,
  `SysFileExists`, `SysIsFileLink`, `SysIsFile`, `SysIsFileDirectory`, `SysRmDir`,
  `SysFileDelete`, `SysFileSearch`, `SysSearchPath`, `SysSleep`, `SysFileMove`, `SysFileCopy`,
  `SysTempFileName`, `SysFormatMessage` (28 unique names).
- A platform-supplied `#include` of `SysRexxUtilFunctions.h` — **the file that answers "what does
  this platform add"**, `interpreter/platform/unix/SysRexxUtilFunctions.h` on Linux: eleven
  semaphore functions gated `#if !defined __APPLE__` (`:3-15`), then unconditionally `SysSetPriority`,
  `SysFork`, `SysWait`, `SysCreatePipe`, `SysCls`, `SysGetKey`, `SysGetMessage`, `SysGetMessageX`,
  `SysMkDir` (`:16-24`), then `SysLinVer` **mapped to the same C function as `SysVersion`** under
  `#ifdef LINUX` (`:25-27`) — `INTERNAL_ROUTINE(SysLinVer, SysVersion)`, i.e. two Rexx-visible names
  for one C++ entry point, which is why §2.2 measured byte-identical output for both — then
  `SysVersion`, `SysSetFileDateTime`, `SysGetFileDateTime`, `SysQueryProcess`, `SysGetErrorText`
  (`:28-32`). This whole file is the committed source for "which names exist on this platform"; the
  Windows sibling at `interpreter/platform/windows/SysRexxUtilFunctions.h` is the same mechanism for
  the 30 Windows-only names in §1.

### 3.2 Name resolution — measured order confirmed against the code

`PackageManager::resolveRoutine(RexxString *function)` (`PackageManager.cpp:347-359`, the
no-package-name overload used for an ordinary external call):
```cpp
RoutineClass *func = getLoadedRoutine(function);   // checks packageRoutines — every loaded package,
                                                    // REXX and REXXUTIL included, built at startup
if (func != OREF_NULL) return func;
return createRegisteredRoutine(function);          // RXAPI-registered fallback, then external file
```
`getLoadedRoutine` succeeding is exactly what §2.1's three probes measured: a call to `SysFileExists`
resolves here, ahead of `createRegisteredRoutine` (the RXAPI/`RxFuncAdd` path) and ahead of the
external-file search that happens further up the call chain when even that fails. Local labels and
`::ROUTINE`s are resolved earlier still, in the language parser/activation before an external call
is even attempted, which is why they shadow both `REXX` and `REXXUTIL` uniformly (§2.1). This is the
identical mechanism survey D found for `DIRECTORY`/`FILESPEC`/`BEEP` — both internal packages are
peers in the same `packages`/`packageRoutines` tables, so a Rust port's internal-package table (D's
proposal, `run.rs:3549`) is architecturally the right place for RexxUtil's names too, not a separate
mechanism.

### 3.3 Per-function implementation

**`SysFileExists` / `SysIsFile`** — both are thin wrappers over `SysFileSystem::exists`/an `access`
or `stat`-based check in `interpreter/platform/unix/SysFileSystem.cpp` (not fully read; the
behavioural difference between the two — directories count for `SysFileExists`, not for
`SysIsFile` — was established by measurement in §2.2, sufficient for a Rust port's test to pin).

**`SysFileDelete`** (`RexxUtilCommon.cpp:1689-1694`) delegates to
`SysFileSystem::deleteFile` (`interpreter/platform/unix/SysFileSystem.cpp:786-793`):
```cpp
int SysFileSystem::deleteFile(const char *name)
{
    if (!canWrite(name)) return EACCES;         // pre-check, unconditional
    return unlink(name) == 0 ? 0 : errno;
}
```
where `canWrite` (`:?`) is `access(name, W_OK) == 0`. **This is the source of §2.2's `13` for a
missing file**: `access` on a nonexistent path fails (correctly, for a different reason than
`EACCES` — `ENOENT`), but the pre-check collapses every `access` failure to the single constant
`EACCES` (13) rather than propagating the real reason. A Rust port that calls `std::fs::remove_file`
directly and maps *its* `io::Error` would get `ENOENT` (2) for a missing file and diverge from the
oracle. This is the same shape survey B flagged in its D6 for `.File`'s `delete`/`renameTo` — a
multi-step sequence the oracle takes that a direct `std::fs` call does not reproduce — extended here
to a second, independent call site with a different (and more surprising) wrong-errno-on-purpose
result.

**`SysRmDir`** (`RexxUtilCommon.cpp:1672-1676`) delegates to `SysFileSystem::deleteDirectory`
(`unix/SysFileSystem.cpp:803-806`): `return remove(name) == 0 ? 0 : errno;` — no pre-check, so
§2.2's `2` (missing) and `39` (non-empty) are the literal `remove(2)` errno, unlike `SysFileDelete`.

**`SysMkDir`** is **not** shared/common code — it has its own Unix-specific
`RexxRoutine2(int, SysMkDir, CSTRING, path, OPTIONAL_int32_t, mode)` in
`interpreter/platform/unix/SysRexxUtil.cpp:576-586`:
```cpp
if (argumentOmitted(2)) mode = S_IRWXU | S_IRWXG | S_IRWXO;   // default 0777, umask still applies
return mkdir(qualifiedName, mode) == 0 ? 0 : errno;
```
raw `mkdir(2)` with the raw errno on failure — matches §2.2 exactly (`17`/`EEXIST` for an existing
name, `mode` applied verbatim modulo `umask`). The common `SysFileSystem::makeDirectory`
(`unix/SysFileSystem.cpp:1102-1105`) exists too but is used by `.File`'s native side, not by
`SysMkDir` — it returns only a bool, losing the errno, so it is not the function to share for error
codes (see §5).

**`SysSleep`** (`RexxUtilCommon.cpp:1889-1919`, platform-independent) validates with
`context->ObjectToDouble`, raising `88.902` for a non-numeric argument and `88.907` for
`seconds < 0.0 || seconds > 2147483.0` — **this bound is hardcoded the same on every platform**; the
Unix-specific `999999999` in the doc (`rexxutil.xml:3423`) does not correspond to anything the C++
enforces. Converts to microseconds and calls `SysThread::longSleep(microseconds)`
(`common/platform/unix/SysThread.cpp:191-201`), which on Linux is `nanosleep(2)` — **a call that
blocks only the calling thread**, matching architecture fact 1 (`SysSleep` need only block the
interpreter's own thread) with no extra work required beyond calling `std::thread::sleep`.

**`SysVersion`** (`unix/SysRexxUtil.cpp:592-609`, Unix-specific): `uname(&info)`, then
`snprintf("%s %s", info.sysname, info.release)`. `SysLinVer` is the identical entry point (§3.1) —
not a separate function.

**`SysFileTree`** (`RexxUtilCommon.cpp:287-303`, `TreeFinder` class spanning `:243-770`+, common
code with a platform-specific inner scan) is the largest of the nine:
- `TreeFinder::findFiles()` (`:339-350`) calls `getFullPath()` then `recursiveFindFile(filePath)`.
- `validateFileSpec()` (`:378-396`) rejects an empty spec (`nullStringException`, error 43.1-shaped
  per-argument), then platform-adjusts trailing `/`/`.`/`..` into a `*` wildcard.
- `recursiveFindFile` (`:662-732`) uses a `SysFileIterator` (platform class, `opendir`/`readdir`
  under the hood) that explicitly skips `"."` and `".."` (`:673-676`) — matches survey B's
  `std::fs::read_dir` observation that Rust's own iterator already excludes both, for free.
  Non-recursive matches are checked and added to the stem as they are found; when `S` is set, a
  **second** iterator pass over the same level (unfiltered by the name pattern, filtered to
  directories only) recurses one level at a time (`:690-729`) — so `S`'s depth-first order in
  §2.2 (top level's files, then the first subdirectory's) is structural, not incidental.
- **Glob matching is `fnmatch(3)`**, confirmed by `#include <fnmatch.h>` at
  `interpreter/platform/unix/SysRexxUtil.cpp:161` (the only file that includes it) — this is exactly
  the POSIX bracket-expression grammar the doc describes (`*`, `?`, `[...]` with `-` ranges and `!`
  negation, case-fold controlled by the `I` option), not a hand-rolled matcher. A Rust port has to
  either match `fnmatch`'s specific bracket semantics (the `glob` crate's `Pattern` is close but its
  negation character and edge-case handling were not independently checked here — see §5) or call
  through `libc::fnmatch` (an `unsafe` FFI site, needing per-site approval under this workspace's
  `unsafe_code = "forbid"`).
- The default-format stem line's exact column layout (§2.2) was not traced to its `sprintf`/format
  string in the C++ — established by measurement only, sufficient to pin a byte-for-byte Rust
  implementation without also reading the formatting code.

### 3.4 The shared path-qualification seam

Every one of these routines that takes a path argument goes through
`RoutineQualifiedName` (`interpreter/memory/ExternalFileBuffer.hpp:129-148`):
```cpp
RoutineQualifiedName(RexxCallContext *c, const char *name) : qualifiedName(c)
{
    SysFileSystem::qualifyStreamName(name, qualifiedName);
}
```
— **the identical function survey B named for `.File`'s own path qualification**
(`SysFileSystem::qualifyStreamName` → `canonicalizeName`, B's D1). `SysFileExists`, `SysFileDelete`,
`SysRmDir`, `SysMkDir`, `SysFileTree`'s file spec, and `.File`'s constructor all resolve a relative
argument through the same one C++ function and therefore the same one process current directory.
This is the strongest evidence in this survey for B's D2 (the shadow current directory is not
`.File`'s to own) applying to RexxUtil too: a `Sys*` function's path argument and `.File`'s own path
must resolve against the *same* shadow-cwd field in the crate, or the two will disagree with each
other the moment a program calls `DIRECTORY(newdir)` and then mixes `SysFileExists` with `.File`
calls in the same program.

## 4. The crate today

**No RexxUtil surface exists at all.** `/bin/grep -rn "Sys" rust/crates/rexx-inventory/src/*.rs`
finds nothing; the crate's builtin machinery (`rust/crates/rexx-exec/src/builtin.rs`) is a
different, unrelated table (see below), and none of its rows name a `Sys*` function.

**Resolution seam** (survey D's finding, independently confirmed by reading the same code): the
call name resolver in `rust/crates/rexx-exec/src/run.rs` (the `resolve_search`-adjacent match
block, `:3530-3577`) tries, in order:
1. A label in scope (`Resolved::Label`).
2. `builtin::resolve(name)` — the true BIF table (`rust/crates/rexx-exec/src/builtin.rs:605-618`),
   which is the crate's model of `BuiltinFunctions.cpp`'s `BUILTIN()` table, not RexxUtil.
3. `builtin::is_excluded_builtin(name)` — a Phase-4-excluded BIF, loud.
4. `self.installed_routine(name)` — a `::ROUTINE` in the running package.
5. `run.rs:3549`, `None if self.library_bootstrap && let Some(program) = rexx_lib::lookup(...)`
   — the embedded `CoreClasses.orx` lookup survey D's report cites for the exact same line number.
6. Fall through to `Raised::routine_not_found(name)` — **43.1, rc 213** (§2.3's measured refusal
   for every one of the 12 sampled names, RexxUtil's nine included).

There is **no step between (4) and (6)** for an always-loaded internal package — which is exactly
what §3.1/§3.2 shows both `REXX` (`DIRECTORY`/`FILESPEC`/`BEEP`) and `REXXUTIL` are on the oracle.
Survey D's proposed fix (an internal-package table consulted at this exact seam, ahead of (5)'s
file search — matching §2.1's measured resolution order) is the right shape for RexxUtil's table
too; nothing here contradicts it, and §3.2 gives the additional oracle-side evidence
(`PackageManager`'s two `loadInternalPackage` calls being peers) that a single table shape can serve
both packages rather than needing two different mechanisms.

**The builtin dispatch machinery** (`rust/crates/rexx-exec/src/builtin.rs:552-640`) is a flat
`IMPLEMENTED: &[Builtin]` array of `{name, min, max, run}` rows, hashed once into a `HashMap` by
`rows()`, with `is_builtin`/`is_excluded_builtin` backed by `rexx_inventory::builtins`'s
in-scope/excluded sets (`rust/crates/rexx-inventory`). This is architecturally the *wrong* place
for RexxUtil: those names are not `BuiltinFunctions.cpp` BIFs, resolve later than builtins on the
oracle (§3.2), and `rexx_inventory::builtins` names only the fixed BIF set — extending it would
misrepresent `SysFileExists` et al. as builtins the way `DIRECTORY` is not one either (survey D's
§4.3). The seam is the resolver step named above, with its own new internal-package table (module
suggestion: alongside wherever survey D's `REXX`-package table for `DIRECTORY`/`FILESPEC`/`BEEP`
lands — the two tables are peers on the oracle and should probably be siblings in the crate, even if
kept as two `HashMap`s for the two different implementation bodies).

**Argument/arity checking**: `builtin.rs`'s `check_arity` (BIF-shaped, `min`/`max` counts) is not
the right model either — §2.2 measured RexxUtil's own arity errors as `88.901`/`88.922`
(native-routine shape, matching survey D's note on `DIRECTORY`/`FILESPEC`/`BEEP`'s arities), not the
`40.x` shape BIFs raise. A `Sys*` implementation needs the `88.9xx` family already used somewhere in
this crate's native-routine handling (not located in this survey; likely near wherever `::ROUTINE`
argument checking or `DIRECTORY`'s own arity lives, per survey D's §2.8/§4.3).

**Shadow filesystem state**: confirmed independently (§3.4) that this crate has none yet — survey B
already established "no shadow current directory exists in the crate today"
(`rust/crates/rexx-exec` — `require.rs:135`'s `normalize(path, cwd)` and the `DIRECTORY()` BIF
placeholder both read the real process cwd). Every one of the nine subset functions that takes a
path (`SysFileExists`, `SysFileDelete`, `SysIsFile`, `SysRmDir`, `SysMkDir`, `SysFileTree`) needs
that same not-yet-built field (architecture fact 1), which makes the shadow-cwd field a prerequisite
shared with `.File` and `DIRECTORY`, not something a RexxUtil task can build in isolation.

## 5. Design questions

**DQ1 — Where does the RexxUtil name table live, and is it one table with `REXX`'s or two?**
§3.2 shows the oracle treats `REXX` (`DIRECTORY`/`FILESPEC`/`BEEP`, survey D) and `REXXUTIL` as two
peer internal packages consulted by the same `getLoadedRoutine` call, at the same point ahead of the
external-file search. Options: **(a)** one shared internal-package `HashMap<&[u8], InternalRoutine>`
at `run.rs:3549` covering both packages' names, dispatching to whichever body the entry names; **(b)**
two separate maps/checks at the same seam. Recommend **(a)**: it is one lookup instead of two on
every unresolved-external-call path (this crate already optimises the equivalent BIF lookup to one
hash, `builtin.rs:610-613`'s own comment says why), and it makes "which internal package" a property
of the table row rather than of which `if` branch ran. Cost if wrong: two near-identical tables that
can drift out of sync on the shared behaviours (both must reject a local-label/`::ROUTINE` shadow the
same way, §2.1) — cheap to fix later by merging, so this is a style question, not a correctness one.

**DQ2 — One shared path-qualification/shadow-cwd helper for `Sys*`, `.File`, and `DIRECTORY`.**
§3.4 measured that `SysFileExists`, `SysFileDelete`, `SysIsFile`, `SysRmDir`, `SysMkDir`, and
`SysFileTree`'s file spec all resolve a relative argument through the identical oracle function
`.File`'s constructor uses (survey B's D1). Architecture fact 1 requires a shadow current directory
per interpreter (not the real process cwd, for thread-safety); survey B's D2 already rules the
shadow-cwd field itself is not `.File`'s to own. Options: **(a)** one Rust module (path
qualification + shadow-cwd accessor) that `.File`'s native entries, the `Sys*` family, and
`DIRECTORY`/`QUALIFY` all call; **(b)** let each area grow its own and reconcile later. Recommend
**(a)**, matching B's own recommendation; cost if wrong is exactly what B already named — `.File`
and a `Sys*` function disagreeing about "the current directory" the moment a program mixes them
after a `DIRECTORY(newdir)` call, which no corpus witness would catch until someone writes exactly
that program. This survey adds no new information to B's D1/D2 beyond confirming RexxUtil is a third
consumer, not a reason to reopen them.

**DQ3 — Reproduce `SysFileDelete`'s `EACCES`-on-missing quirk, or fix it?** §3.3/§2.2 measured the
oracle answering `13` (not `2`) for `SysFileDelete` on a nonexistent file, because
`SysFileSystem::deleteFile` pre-checks `access(name, W_OK)` and returns the constant `EACCES` for
*any* pre-check failure, masking the real reason. Options: **(a)** reproduce byte-for-byte — call
`access(2)` first (via `nix::unistd::access` or `rustix::fs::access`, both safe wrappers, no
`unsafe` needed) and return the fixed code on failure before ever calling `std::fs::remove_file`;
**(b)** call `std::fs::remove_file` directly and map its `io::Error::raw_os_error()`, which would
give `2` for this case and diverge. Recommend **(a)**: parity with the oracle is this project's
standing goal (`[[oorexx-rust-perf-diagnosis]]`), the quirk is cheap to reproduce once identified,
and it is exactly the shape survey B's D6 already flagged for `renameTo`/`delete` — this is
`SysFileDelete` needing the same discipline, not a new problem. Cost if wrong: a silent divergence
on one specific error code, invisible until a corpus program calls `SysFileDelete` on a missing file
and checks the return value, which is exactly the "one send to a fresh receiver" failure shape this
project has already been bitten by once.

**DQ4 — `SysRmDir`/`SysMkDir`/`SysFileDelete`'s "success" errno numbers.** §2.2 measured raw Linux
errno values (`2`=`ENOENT`, `17`=`EEXIST`, `21`=`EISDIR`, `39`=`ENOTEMPTY`) for the non-pre-checked
cases. `std::io::Error::raw_os_error()` on Linux is a thin wrapper over the same `errno` the syscall
sets, so `std::fs::remove_dir`/`create_dir`/`remove_file`'s own `io::Error` already carries the right
number for every case *except* the `EACCES`-on-missing one DQ3 covers — no new dependency needed for
this half, only for the explicit `access(2)` pre-check. Not itself risky; named here because it is
easy to over-solve (reach for `nix`/`rustix` everywhere) when `std::fs` alone is already correct for
three of the four measured cases.

**DQ5 — `SysFileTree`'s glob matching is POSIX `fnmatch(3)` (§3.3), and the workspace forbids
`unsafe`.** `libc::fnmatch` would need an `unsafe` FFI call, needing Moritz's per-site approval.
Options: **(a)** the offline-registry `glob` crate (0.3.3/0.3.4) — its `Pattern` matcher is close to
shell/`fnmatch` globbing, but this survey did **not** verify its bracket-negation character (`!` per
the doc/POSIX) or its exact edge-case behaviour (a lone `[`, an empty bracket, a `-` at the start or
end of a class) against `fnmatch(3)`'s actual behaviour — that comparison is unstarted work, not a
decision; **(b)** hand-port the specific bracket grammar `SysFileTree` exercises (`*`, `?`,
`[abc]`, `[!abc]`, `[a-z]`), which is small and fully specified by the doc, avoiding both `unsafe`
and a dependency whose semantics were not checked. Recommend starting with **(b)** for exactly this
reason — a hand-port's behaviour is fully known before it ships, where crate-compatibility is a
question a task would still have to answer — and revisiting **(a)** only if the hand-port grows past
what the doc's grammar actually needs. Cost if wrong: a silent difference in bracket-class matching
that only a targeted witness (a filename that exercises `[a-z]` or `[!...]`) would catch, so that
witness belongs in the task slice regardless of which option is taken (§6, Task 3).

**DQ6 — `SysSleep`'s real bound is `2147483`, the doc's `999999999` is stale for Unix (§2.2/§3.3).**
The crate should validate against the number the oracle actually enforces (`2147483.0`, same on
every platform per the C++), not the documented one, or a corpus witness at any value between the
two would diverge. Low cost either way to fix later, but cheap to get right now that it is measured;
flagged so a future reader trusts this survey over the doc on this one number.

**DQ7 — `SysVersion`/`SysLinVer` are host-dependent (§2.2), and the differential still works.**
Because the corpus gate runs the crate and the oracle **on the same host** (`rust/CLAUDE.md`'s own
"The oracle" section — the same machine, not a fixed golden file), a Rust `uname(2)` call (`nix`'s
`sys::utsname::uname()`, a safe wrapper, or `std::env::consts` is not enough — it does not expose
the kernel release string) will match the oracle byte-for-byte on whatever machine the gate runs on,
same as any other host-dependent value this corpus already carries. The only thing to get right is
not hardcoding today's string anywhere (as a doc comment already nearly does, `rexxutil.xml`'s own
examples) and calling the real syscall. No `unsafe` needed (`nix::sys::utsname` is a safe wrapper
over `uname(2)`).

**DQ8 — `SysMkDir`'s Unix `mode` argument** needs the raw permission bits applied before `umask`,
exactly as `mkdir(2)` does (§3.3/§2.2). `std::os::unix::fs::DirBuilderExt::mode` (stdlib, `unix`
target only, no new dependency, no `unsafe`) sets exactly this. Low risk; named because a naive
`std::fs::create_dir` (no `DirBuilderExt`) would silently ignore the second argument.

**DQ9 — Loud refusal for the Phase 10 remainder, and for the crate's three internal-`REXX`-package
routines survey D found (`DIRECTORY`/`FILESPEC`/`BEEP`).** Today all of these answer 43.1/rc 213 —
a normal, trappable "no such routine", indistinguishable from a program's own typo (§2.3). The
brief and survey D both want a louder, more honest refusal once the resolver in DQ1 exists to carry
it. Options: **(a)** a single derived, committed name list — every name in §3.1's full registration
table (both `rexxutil_routines`'s explicit list and its Unix `#include`, i.e. every Linux-registered
`Sys*` name, including the 30 Windows-only doc'd names for when they're called on Linux, the 7
macrospace names, and the rxunixsys/other-library names are explicitly **not** on this list because
they are not RexxUtil's problem) minus the 9 the plan implements, checked at DQ1's same resolver seam
right after the internal-package hit-or-miss, before falling to `routine_not_found`; **(b)** leave
them all falling through to 43.1 until each is implemented. Recommend **(a)**, matching D-P7-5's
already-taken decision for excluded builtins ("the excluded-builtin owner message is fixed... with
an assertion so it cannot drift back") — the same discipline applies here, and the same drift risk
(`4c` on all fifteen excluded builtins) is what happens without it. The list is derived, not typed by
hand: `interpreter/runtime/RexxUtilCommon.cpp`'s explicit table plus `SysRexxUtilFunctions.h`'s
platform `#include`, both cited by line above, are the two files an assertion or test can re-read
to keep the list from drifting. Cost if skipped: exactly D-P7-5's cost, paid twice (once for BIFs,
once here) — cheap now, invisible-forever if deferred.

**No `unsafe` is needed anywhere in this subset.** Every one of the nine functions maps to either
plain `std::fs`/`std::os::unix::fs` (DQ4, DQ8), a safe wrapper crate already in the offline registry
(`nix` or `rustix` for `access(2)` in DQ3, `nix::sys::utsname` for DQ7), or a hand-port with no FFI
(DQ5). The one place a naive implementation would reach for `unsafe` — calling `libc::fnmatch`
directly for `SysFileTree` — has a safe alternative (DQ5) that this survey recommends specifically
to avoid needing Moritz's per-site sign-off.

## 6. Proposed task slices

(pending)

## 7. Not done / not established

(pending)
