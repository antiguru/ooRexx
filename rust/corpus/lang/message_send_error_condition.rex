/* A synchronous send's error trapped in its caller: ~errorCondition is the
   condition object, and ~result re-raises it with its whole traceback. */
o = .t~new
m = .message~new(o, 'boom', 'I', 7)
call doit m
say 'hasError' m~hasError m~completed m~hasResult
c = m~errorCondition
say 'position' c~position 'traceback' c~traceback~items 'frames' c~stackFrames~items
do line over c~traceback; say '>' line; end
r = m~result
exit
doit: procedure
  use arg m
  signal on syntax
  m~send
  return
syntax: say 'trapped' condition('O')~traceback~items; return
::class t
::method boom
  use arg n
  return n / 0
