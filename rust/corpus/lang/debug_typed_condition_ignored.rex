/* A NOTREADY raised by a line typed at a pause is not trapped. */
call on notready name h
trace ?a
x = linein('/nonexistent/aa')
say 'after'
exit
h: say 'handler sigl' sigl condition('D'); return
