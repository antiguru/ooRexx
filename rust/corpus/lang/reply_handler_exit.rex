/* A CALL ON handler run at the REPLY clause's end that ends with EXIT ends
   the method before its split: the sender receives no result. */
.local~main = .context~thread
say 'got' .k~new~m
call SysSleep 0.2
say 'main end'
::class k
::method m
  call on notready name nr
  reply 'v' || linein('/nonexistent/file/x')
  say 'rest on main' (.context~thread = .local~main)
  return
nr:
  say 'handler on main' (.context~thread = .local~main)
  exit
