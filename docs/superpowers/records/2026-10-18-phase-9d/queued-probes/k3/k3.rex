o = .obj~new
o~u
o~r
say 'end' result
call syssleep 0.3
exit
::class obj
::method u
  expose w
  trace ?a
  w = 1
  guard on
  guard off when w = 1
  return
::method r
  trace ?a
  reply 5
  say 'after reply'
  nop
  return
