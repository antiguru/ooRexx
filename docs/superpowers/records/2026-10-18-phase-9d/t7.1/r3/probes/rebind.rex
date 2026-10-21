/* rebind the name to a fresh copy each pass; one call site (in cy/cx) */
do i = 1 to 3
  d = .routines~r~copy
  .routines['Y'] = d
  call cy 'routines' i, d
end
do i = 1 to 3
  c = .routines~r~copy
  .context~package~addRoutine('X', c)
  call cx 'addRoutine' i, c
end
.routines['Y'] = .routines~r
call cy 'routines canon', .routines~r
exit 0
cy: procedure
  use arg tag, d
  signal on syntax name s
  call y 93900
s: call show tag, condition('O'), d
  return
cx: procedure
  use arg tag, d
  signal on syntax name s1
  call x 93900
s1: call show tag, condition('O'), d
  return
::routine show
  use arg tag, condition, expected
  do f over condition~stackFrames
    e = f~executable
    if f~type == 'ROUTINE' then say tag f~name (e == expected) (e == .routines~r) e~class~id
  end
::routine r external 'LIBRARY orxfunction TestRaiseException0'
