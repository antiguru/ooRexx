# STREAM(name, 'C', 'OPEN READ') on a missing file raises 91.999

Found 2026-10-16 by an external-corpus probe (rx2cpp fixtures, scratch only). Re-run by the
controller, fresh empty dir, 9b1 close binary `ooRexx-p9-pin/rexx-run-9b1-d58e12600`:

    say stream('zz_missing.dat','C','OPEN READ')
    say 'rc ok'

Oracle: `ERROR:2`, `rc ok`, rc 0. Crate: Error 91.999 `Message "COMMAND" did not return a result.`,
rc 165. The object path `.stream~new(name)~open('READ')` answers `ERROR:2` on both, so the BIF's
COMMAND route drops the open's result on failure. A wrong answer, not a refusal.
