/* The trace of USE ARG into message and bracket terms: the argument's
   result, then the term's own arguments and its assignment. */
o = .t~new
d = .directory~new
call sub 'x', 'y'
call sub2
exit
sub:
trace r
use arg o~a, d['K']
trace i
use arg o~a, d['K']
trace o
return
sub2:
trace i
use arg o~a = 'dflt', d['Z'] = 'e'
trace o
say o~a d['Z']
return
::class t
::attribute a
