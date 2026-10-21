c = .routines~r~copy
signal on syntax name s5
c~call(93900)
s5: call show 'copy', condition('O'), c
m = .Routine~new('m', 'call r arg(1)')
signal on syntax name s6
m~call(93900)
s6: call show 'viam', condition('O'), c
exit 0
::routine r external 'LIBRARY orxfunction TestRaiseException0'
::routine show
  use arg tag, cond, c
  do f over cond~stackFrames
  e = f~executable
  say tag f~type f~name e~class~id (e == c) (e == .routines~r)
  end
