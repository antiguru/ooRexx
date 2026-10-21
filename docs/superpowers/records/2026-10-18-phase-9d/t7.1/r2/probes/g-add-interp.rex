parse arg n
if n = '' then n = 100000
do i = 1 to n
  d = .routines~i~copy
  .context~package~addRoutine('Y', d)
  interpret 'call y 1'
end
say 'done' n result
::routine i external 'LIBRARY orxfunction TestIntArg'
