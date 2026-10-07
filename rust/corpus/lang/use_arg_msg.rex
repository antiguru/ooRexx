/* USE ARG into a message term sends the term's NAME= with the argument; an
   omitted argument without a default leaves the term alone. */
o = .t~new
call sub 'x'
say o~a
call sub
say o~a
exit
sub: use arg o~a; return
::class t
::attribute a
