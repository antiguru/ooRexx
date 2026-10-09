/* EXIT with a value in a CALL ON handler run at the REPLY clause's end is
   98.937, raised to the sender. */
say 'got' .k~new~m
call SysSleep 0.2
say 'main end'
::class k
::method m
  call on notready name nr
  reply 'v' || linein('/nonexistent/file/x')
  say 'rest'
  return
nr:
  say 'handler'
  exit 'x'
