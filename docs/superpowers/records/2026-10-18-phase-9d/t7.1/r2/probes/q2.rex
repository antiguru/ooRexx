do k = 1 to 3
  if k = 2 then .routines['TESTINTARG'] = .routines~r
  call ct k
end
exit 0
ct: procedure
  use arg k
  signal on syntax name s
  call TestIntArg 7
  say k 'answered' result
  return
s: say k 'raised' condition('O')~code
  return
::routine r external 'LIBRARY orxfunction TestRaiseException0'
