.routines['Y'] = .routines~i~copy
.local~hash = .routines['Y']~identityHash
call cy 1
.routines['Y'] = .routines~r
do k = 1 to 300000
  a = .array~of(k, k || 'x', .directory~new)
end
call cy 'abc'
exit 0
cy: procedure
  use arg v
  signal on syntax name s
  call y v
  say 'answered' result
  return
s: c = condition('O')
  say 'raised' c~code
  do f over c~stackFrames
    e = f~executable
    if f~type == 'ROUTINE' then say 'frame' f~name e~class~id (e == .routines~i) (e == .routines~r) (e~identityHash == .hash)
  end
  return
::routine i external 'LIBRARY orxfunction TestIntArg'
::routine r external 'LIBRARY orxfunction TestRaiseException0'
