c = .routines~r~copy
signal on syntax name s1
y = c~call(.w~new)
s1: say 'raised' condition('O')~code
exit 0
::routine r external 'LIBRARY orxfunction TestNestedAttach'
::class w
::method string
  do f over .context~stackFrames
    e = f~executable
    if f~type \== 'PROGRAM' then say 'live' f~type f~name (e == .routines~r) e~class~id
  end
  return 'w'
