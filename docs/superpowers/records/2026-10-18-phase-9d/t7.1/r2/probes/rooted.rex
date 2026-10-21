/* a site runs a copy, the copy is dropped everywhere, the heap churns, and
   the same site runs again: the frame must still name the routine it ran */
.routines['Y'] = .routines~r~copy
call cy 'first'
.routines['Y'] = .routines~i
do k = 1 to 300000
  a = .array~of(k, k || 'x', .directory~new)
end
call cy 'second'
exit 0
cy: procedure
  use arg tag
  signal on syntax name s
  call y 93900
  say tag 'no raise' result
  return
s: c = condition('O')
  do f over c~stackFrames
    if f~type == 'ROUTINE' then say tag 'raised' f~name f~executable~class~id
  end
  return
::routine r external 'LIBRARY orxfunction TestRaiseException0'
::routine i external 'LIBRARY orxfunction TestIntArg'
