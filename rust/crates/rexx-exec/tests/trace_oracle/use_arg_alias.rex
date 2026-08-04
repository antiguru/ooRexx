/* >R>: USE ARG > aliasing, under TRACE R -- a RESULTS-level prefix. */
trace r
orig = 'before'
call sub >orig
say orig
exit
sub:
procedure
use arg >qq
qq = 'after'
return
