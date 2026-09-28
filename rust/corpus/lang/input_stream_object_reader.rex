-- ADDRESS WITH INPUT USING an InputStream reads LINEIN until a send of it
-- raises a condition: NOTREADY at the end, any other condition likewise,
-- none of them reaching the issuing activation's traps; a SYNTAX condition
-- is raised once the command has completed.
out = .array~new
address system 'cat' with input using (.AIS~new(('Line1', '', 'Line3'))) output using (out)
say 'shell' rc out~items out~makestring('l', '|')
call on notready name nr
address system 'cat' with input using (.AIS~new(('a', 'b'))) output using (out)
say 'callon' rc out~items
call off notready
address system 'cat' with input using (.Boom~new) output using (out)
say 'user' rc out~items out~makestring('l', '|')
address system 'cat' with input using (.Nested~new) output using (out)
say 'nested' rc out~items out~makestring('l', '|')
call TestAddCommandEnvironment "io", "redirecting"
address io 'INPUTOUTPUT' with input using (.AIS~new(('Line1', 'Line2'))) output using (out)
say 'handler' rc out~items out~makestring('l', '|')
signal on syntax
address system 'cat' with input using (.Bad~new) output using (out)
say 'no condition'
exit
nr: say 'notready handler ran' condition('D'); return
syntax:
  c = condition('O')
  say 'trapped' c~code out~items rc c~position filespec('n', c~program)
  do l over c~traceback; say ' ' l; end
  address system 'cat' with input using (.Bad~new) output using (out)
  exit
::routine TestAddCommandEnvironment EXTERNAL "LIBRARY orxfunction TestAddCommandEnvironment"
::class AIS inherit InputStream
::method init
  expose lines current
  use strict arg lines
  current = 1
::method linein
  expose lines current
  if current > lines~items then raise notready additional(self) return("")
  current += 1
  return lines[current - 1]
::class Boom inherit InputStream
::method init
  expose n
  n = 0
::method linein
  expose n
  n += 1
  if n > 1 then raise user stop return('x')
  return 'first'
::class Nested inherit InputStream
::method init
  expose n
  n = 0
::method linein
  expose n
  n += 1
  if n > 2 then raise notready return('')
  return self~inner(n)
::method inner
  use arg k
  if k = 2 then raise user deep return('from inner')
  return 'n' || k
::class Bad inherit InputStream
::method init
  expose n
  n = 0
::method linein
  expose n
  n += 1
  if n > 1 then return 1 + 'a'
  return 'first'
