parse arg n k
.routines['Y'] = .routines~i~copy
if k == 'canon' then .routines['Y'] = .routines~i
do j = 1 to n
  call y 1
end
say 'done' n k
::routine i external 'LIBRARY orxfunction TestIntArg'
