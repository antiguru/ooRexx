-- A SYNTAX condition a callback raised inside an extension's method is
-- re-raised in the method's caller: POSITION and the report's line are the
-- caller's, the traceback keeps the raising line.
k = .k~new
signal on syntax
x = k~send0(.t~new, 'boom')
exit
syntax:
  c = condition('O')
  say c~code c~position filespec('n', c~program) c~traceback~items
  do l over c~traceback; say ' ' l; end
  signal off syntax
  y = k~send0(.t~new, 'boom')
::class k
::method send0 external "LIBRARY orxmethod TestSendMessage0"
::class t
::method boom
  nop
  return 1 + 'a'
