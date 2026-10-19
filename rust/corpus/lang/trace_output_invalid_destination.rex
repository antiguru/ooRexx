/* A .TRACEOUTPUT destination with no LINEOUT: the failed delivery raises
   97.1 in the traced clause, which does not run, and the caller's trap takes
   it. The handler pops the destination. */
signal on syntax name s1
call r1
say 'not reached'
s1:
  call popped
signal on syntax name s2
call r2
say 'not reached'
s2:
  call popped
signal on syntax name s3
.o~new~m
say 'not reached'
s3:
  call popped
signal on syntax name s4
code = .routine~new('rr', .array~of('.traceOutput~destination("")', "say 'rr say ran'", '::options trace r'))
.traceOutput~destination(.shown~new)
code~call
say 'not reached'
s4:
  call popped
x = .traceOutput~destination
say 'done'
exit
popped:
  c = condition('o')
  x = .traceOutput~destination
  say c~code c~message
  return
::routine r1
trace r
.traceOutput~destination("")
say 'r1 say ran'
::routine r2
.traceOutput~destination("")
a = 1
trace r
a = 2
say 'r2 a' a
::class o
::method m
  expose v
  .traceOutput~destination("")
  trace r
  trace n
  say 'm ended'
::class shown
::method lineout
  use arg traced
  line = traced~makeString
  at = pos(' Routine "', line)
  if at > 0 then line = left(line, at + 8)
  at = pos(' in package', line)
  if at > 0 then line = left(line, at - 1)
  say 'trace:' line
  return 0
