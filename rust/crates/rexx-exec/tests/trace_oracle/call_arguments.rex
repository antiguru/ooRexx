/* >A>: one line per argument at the CALL site, omitted positions included. */
trace i
zref = 'refval'
call sub 1+1,,'three',>zref
exit
sub:
use arg aa, bb, cc, dd
say aa bb cc dd
return
