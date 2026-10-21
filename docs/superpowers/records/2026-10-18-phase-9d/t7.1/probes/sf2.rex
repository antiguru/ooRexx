rr = .routines~r
signal on syntax name s1
call r 93900
s1: call show 'a', condition('O')
signal on syntax name s2
rr~call(93900)
s2: call show 'b', condition('O')
signal on syntax name s4
x = r(93900)
s4: call show 'd', condition('O')
signal on syntax name s6
call q
s6: call show 'f', condition('O')
exit 0
::routine show
  use arg tag, c
  do f over c~stackFrames
    e = f~executable
    say tag f~type f~name e~class~id (e == .routines~r) (e == .routines~q)
  end
::routine r external 'LIBRARY orxfunction TestRaiseException0'
::routine q
  call r 93900
