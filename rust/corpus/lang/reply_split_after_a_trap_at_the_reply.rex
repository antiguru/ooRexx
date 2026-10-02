/* A REPLY whose expression raises a condition CALL ON traps: the handler
   runs at the REPLY clause's end on the sender's activity, and the rest of
   the method still runs on the continuation's. */
.local~main = .context~thread
q = .queue~new
say 'got' .k~new~m(q)
do until q~items >= 2; call SysSleep 0.01; end
do while q~items > 0; say q~pull; end
::class k
::method m
  use arg q
  call on notready name nr
  reply 'v' || linein('/nonexistent/file/x')
  q~queue('after reply on main' (.context~thread = .local~main))
  return
nr:
  q~queue('handler on main' (.context~thread = .local~main))
  return
