/* >F>: the expression call form's own value line, at the caller's indent. */
trace i
zz = twice(4) + twice(1)
say zz
exit
twice:
use arg nn
return nn * 2
