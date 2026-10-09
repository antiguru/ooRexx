/* A command typed at a pause raises no ERROR, traces nothing and leaves RC
   alone. */
call on error name h
trace ?a
'false'
say 'after'
exit
sub: say 'in sub' sigl; return
h: say 'handler sigl' sigl rc; return
