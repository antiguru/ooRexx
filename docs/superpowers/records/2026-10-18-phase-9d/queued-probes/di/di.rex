.DebugInput~destination(.lines~new)
trace ?a
nop
say 'after'
exit
::class lines
::method init
  expose n
  n = 0
::method linein
  expose n
  n = n + 1
  if n = 1 then return "say 'typed'"
  return ''
