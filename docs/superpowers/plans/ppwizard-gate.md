# PPWIZARD as a gate -- what works today, what it is blocked on, and by whom

**Added 2026-08-12 at Moritz's request, parked the same day.**
This is a deferred gate: nothing here blocks 4f, and nothing here is wired into a test.
It exists so that the phase that can finally run it does not have to rediscover any of it.

## What it is

PPWIZARD is Dennis Bareis's HTML preprocessor, `https://dennisbareis.com/ppwizard.htm`, written in Rexx.
The cross-platform archive is `https://dennisbareis.com/zips_fw/ppwall.zip`, 122,074 bytes, and `ppwizard.rex` inside it is **17,808 lines and 450,987 bytes** of real-world Rexx, CRLF-terminated, and not valid UTF-8 (there is a `0xC9` byte at offset 249,791 -- read it as bytes).

It is a better gate than anything this project has written for itself, for one reason: nobody wrote it to exercise an interpreter.

## The parse gate, which passes today

`ppwizard.rex` carries its own syntax-check entry on its eleventh line:

```rexx
if arg(1)="!CheckSyntax!" then exit(21924)
```

Rexx parses a program in full before running any of it, so this exercises the whole 451 KB and then exits.
Measured 2026-08-12, each from a fresh empty directory, `ulimit -v 8388608`, absolute paths:

| | oracle | this crate, `REXX_ENGINE=ir` |
|---|---|---|
| exit status | 164 | 164 |
| stdout | empty | empty |
| stderr | empty | empty |

164 is `21924 mod 256`, so both interpreters reached that line and took it.
**This is available now and costs one invocation.**

## Running it end to end: the oracle can, once PPWIZARD's own detection is fixed

The first attempt failed and the first reading of that failure was wrong.
It looked like "PPWIZARD needs Regina", which is what its own web page says.
**It is not: ooRexx runs it, and the blocker was PPWIZARD's 2001-era interpreter detection.**

`RexWhich` has three branches -- `REGINA`, `REXX370`, and otherwise `STANDARD_OS/2`.
ooRexx falls into the last, being OS/2 Rexx's descendant, and that branch then makes two assumptions that a modern ooRexx on Linux does not satisfy.

Two lines fix it, and both are one-word changes:

```diff
-if RexSystemOpSys="BEOS" then
+if RexSystemOpSys="BEOS" | RexSystemOpSys="LINUX" | RexSystemOpSys="AIX" | RexSystemOpSys="DARWIN" then
 RexSystemOpSys="UNIX"

-RexEnvVarPool='OS2ENVIRONMENT'
+RexEnvVarPool='ENVIRONMENT'
```

**The pool name.** Measured on the oracle, one call each under `SIGNAL ON SYNTAX`: `value('HOME',,'ENVIRONMENT')` returns the value; `'OS2ENVIRONMENT'`, `'SYSTEM'` and `'EXTERNAL'` each raise 40 (`40.914`, "Unknown VALUE function variable environment selector").
So ooRexx accepts neither classic OS/2 Rexx's spelling nor Regina's, and PPWIZARD only knows those two.

**The operating system.** ooRexx's `PARSE SOURCE` says `LINUX` where Regina says `UNIX`, and every UNIX-ism in the file keys off that one word -- including `RexDirChar`, which is why the unpatched run reported `No input files matched "/tmp/tmp.XXXX\tryme.it"` with a backslash.

With both applied, on the sample input the archive ships for exactly this purpose:

```
rexx ppwizard-oorexx.rex tryme.it     ->  rc 0, writes tryme.htm
                                          10,638 bytes
                                          sha256 790105dc45f355ae6ab8c17157597e452735bc7c4715134cecd9010b9452eaab
```

**A deterministic output file to hash is what makes this a differential gate** rather than a smoke test.
The hash above is this host, this oracle build, this input; it is a starting point to re-derive, not a constant to trust.

## What this crate is blocked on, and every blocker is already an exclusion

Probed 2026-08-12, one call per line, rather than read off a grep of the source:

| needed by PPWIZARD | this crate | owed by |
|---|---|---|
| `LINEIN` `LINEOUT` `LINES` `CHAROUT` `STREAM` | not implemented (4c) | **Phase 7**, the stream model and platform layer |
| `SETLOCAL` | not implemented (4c) | **Phase 7**, and its row already records the `set_var` unsafety decision |
| `RXFUNCADD` | not implemented (4c) | **Phase 10**, the RXAPI daemon |
| `SYSFILEDELETE` `SYSFILETREE` `SYSTEMPFILENAME` `SYSSLEEP` | not found | the platform layer |
| `FILESPEC` `DIRECTORY` | not found | the platform layer |
| `QUEUED` | works | -- |

We stop at the first `RxFuncAdd`, `ppwizard.rex:426`.

**So the run gate is Phase 10's, not Phase 4's**, on the latest of its blockers, and Phase 7 owes the bulk of the work.
One thing worth flagging for whoever gets there: PPWIZARD's `RxFuncAdd` calls register RexxUtil functions that ooRexx already has built in, so under the oracle they are near no-ops. Whether this crate can answer them the same way is a Phase 10 decision and is not assumed here.

## Licence, which constrains how this gets vendored

`ppwizard.lic`: free for any use including commercial; "This archive may be freely distributed (but not sold) **in full**. Files must not be removed from the distributed exe or zip files"; and explicitly, "**You may modify the code to suit your needs**", which is what the two-line patch above is.

**So committing `ppwizard.rex` on its own would breach the distribution term.**
The two ways that do not: vendor the whole 122 KB `ppwall.zip` plus our patch beside it, or fetch the archive at gate time and apply the patch then.
Not chosen here. A gate that needs the network is fragile, and a repository that carries third-party freeware is a decision rather than a detail.
