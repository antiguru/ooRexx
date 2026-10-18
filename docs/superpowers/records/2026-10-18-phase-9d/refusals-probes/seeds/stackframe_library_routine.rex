-- StackFrame~executable on a library routine's frame, taken from the condition
-- a library routine raises.
signal on syntax
call r 93900
exit 0
syntax:
  do f over condition('O')~stackFrames
    say 'frame' f~type f~name
    e = f~executable
    say 'executable' e~class~id (e == .nil) (e == .routines~r)
  end
  exit 0
::routine r external 'LIBRARY orxfunction TestRaiseException0'
