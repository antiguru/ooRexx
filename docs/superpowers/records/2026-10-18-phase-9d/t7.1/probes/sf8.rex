signal on syntax name s5
call TestRaiseException0 93900
s5: call show 'pkg', condition('O')
signal on syntax name s6
call TestRaiseException0 93900
s6: call show 'pkg2', condition('O')
exit 0
::routine r external 'LIBRARY orxfunction TestRaiseException0'
::routine show
  use arg tag, c
  lr = .Routine~loadExternalRoutine('TestRaiseException0', 'LIBRARY orxfunction TestRaiseException0')
  f = c~stackFrames~firstItem
  e = f~executable
  if tag = 'pkg' then .local~first = e
  say tag f~type f~name e~class~id (e == lr) (e == .routines~r) (e == .first)
