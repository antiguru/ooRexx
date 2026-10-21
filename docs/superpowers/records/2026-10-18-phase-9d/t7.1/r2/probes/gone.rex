/* the copy is held only by the table; the call's argument rebinds the name
   and churns the heap before the first call runs */
.routines['Y'] = .routines~r~copy
.context~package~addRoutine('X', .routines~r~copy)
signal on syntax name s1
call y churn('Y')
s1: call show 'routines', condition('O')
signal on syntax name s2
z = x(churn('X'))
s2: call show 'addRoutine', condition('O')
exit 0
churn: procedure
  use arg n
  if n == 'Y' then .routines['Y'] = .routines~i
  else .context~package~addRoutine('X', .routines~i)
  do k = 1 to 200000
    a = .array~of(k, k || 'x', .directory~new)
  end
  return 93900
::routine show
  use arg tag, condition
  if condition == .nil then do; say tag 'no condition'; return; end
  say tag condition~code
  do f over condition~stackFrames
    if f~type \== 'ROUTINE' then iterate
    ex = f~executable
    say tag f~name ex~class~id (ex == .routines~r) (ex == .routines~i)
  end
::routine r external 'LIBRARY orxfunction TestRaiseException0'
::routine i external 'LIBRARY orxfunction TestIntArg'
