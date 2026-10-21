/* StackFrame~executable on the frame of a copy of a library Routine: a copy
   added under a new name by addRoutine and by a .ROUTINES write, called by
   that name, and a copy's ~call read from a live .context~stackFrames while
   the routine calls back. */
c = .routines~r~copy
.context~package~addRoutine('X', c)
signal on syntax name s1
call x 93900
s1: call show 'addRoutine', condition('O'), c
d = .routines~r~copy
.routines['Y'] = d
signal on syntax name s2
call y 93900
s2: call show 'routines', condition('O'), d
signal on syntax name s3
z = y(93900)
s3: call show 'function', condition('O'), d
.local~copy = .routines~a~copy
signal on syntax name s4
v = .copy~call(.w~new)
s4: say 'attach' condition('O')~code
exit 0
::routine show
  use arg tag, condition, expected
  do f over condition~stackFrames
    e = f~executable
    if f~type == 'ROUTINE' then say tag f~name (e == expected) (e == .routines~r) e~class~id
  end
::routine r external 'LIBRARY orxfunction TestRaiseException0'
::routine a external 'LIBRARY orxfunction TestNestedAttach'
::class w
::method string
  do f over .context~stackFrames
    e = f~executable
    if f~type == 'ROUTINE' then say 'live' f~name (e == .copy) (e == .routines~a) e~class~id
  end
  return 'w'
