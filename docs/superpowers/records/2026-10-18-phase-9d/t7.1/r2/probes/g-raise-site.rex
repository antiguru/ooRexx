parse arg n
if n = '' then n = 100000
do i = 1 to n
  d = .routines~r~copy
  .routines['Y'] = d
  call cy
end
say 'done' n
exit 0
cy:
  signal on syntax name s
  call y 93900
s: return
::routine r external 'LIBRARY orxfunction TestRaiseException0'
