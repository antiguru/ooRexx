# Known test failures

Each file lists tests, as `CLASS/TEST`, that are allowed to fail in CI on a
given platform. `common.txt` applies everywhere and is always loaded alongside
the platform's own file. Matching is case insensitive, because the suite is not
consistent about it: it reports `LINEOUT.TESTGROUP` and
`SysGetXxxPathName.testgroup` where the test files say `.testGroup`.

**An entry has to be a property of the machine, not of ooRexx.** No printer
installed, no console attached, a filesystem that does not generate 8.3 names:
those belong here, with the reason written down. A test that fails because the
interpreter is wrong does not belong here, it belongs in a bug report. Every
entry that goes in without a reason makes the next person assume the rest are
fine too.

Two things are deliberately impossible to excuse from these files:

- **The suite not reaching its summary.** That is how a crash presents, and it
  fails the job whatever the exit code was.
- **A disagreement between the number of failure records parsed and the number
  the suite reported.** Otherwise a change to the report format could make
  everything match nothing and go green.

The suite itself has a separate mechanism, unrelated to this one: a test author
can prefix an assertion message with `.ooRexxUnit.knownBugFlag` to mark a
failure as known, and the suite then keeps it out of its own exit code. That is
for interpreter bugs the project already knows about. These files are only for
things the CI machine causes.

## Wildcards

An entry may name one test as `CLASS/TEST`, or a whole group as `CLASS/*`.

Use the wildcard only for a group that fails intermittently on a different test
each run, where naming tests would neither match reliably nor describe the
problem. It hides every test in that group, so it costs real coverage. Say in
the comment what was measured and, if the cause is in ooRexx rather than in the
machine, say that too and treat the entry as temporary.
