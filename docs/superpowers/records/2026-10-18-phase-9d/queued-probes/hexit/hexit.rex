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
