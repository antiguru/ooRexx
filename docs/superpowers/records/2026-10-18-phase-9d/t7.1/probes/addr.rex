c = .routines~r~copy
.context~package~addRoutine('X', c)
signal on syntax name s1
call x 93900
s1:
do f over condition('O')~stackFrames
  e = f~executable
  if f~type \== 'PROGRAM' then say f~type f~name (e == c) (e == .routines~r)
end
exit 0
::routine r external 'LIBRARY orxfunction TestRaiseException0'
