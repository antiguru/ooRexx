do k = 1 to 3
  if k = 2 then .routines['YY'] = .routines~i
  if k = 3 then .context~package~addRoutine('YY', .routines~i)
  say k yy(7)
end
::routine i external 'LIBRARY orxfunction TestIntArg'
