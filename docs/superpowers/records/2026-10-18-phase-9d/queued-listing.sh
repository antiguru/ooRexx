#!/bin/sh
# Prints the item column of queued-inventory.tsv, in order: every file under
# .superpowers/sdd/queued/, then each carried bullet of the 9a, 9b1 and 9c gate records as
# gate:<record>:<line>. Run from the repository root.
LC_ALL=C ls -1 .superpowers/sdd/queued/
awk '/^## Carried to 9b-9d/{on=1;next} /^## /{on=0} on && /^\* /{print "gate:phase-9a-gate.md:" NR}' docs/superpowers/plans/phase-9a-gate.md
awk '/^## Carried forward/{on=1;next} /^## /{on=0} on && /^- /{print "gate:phase-9b1-gate.md:" NR}' docs/superpowers/plans/phase-9b1-gate.md
awk '/^\*\*Carried, filed during 9c\.\*\*/{on=1;next} /^(## |\*\*)/{on=0} on && /^\* /{print "gate:phase-9c-gate.md:" NR}' docs/superpowers/plans/phase-9c-gate.md
