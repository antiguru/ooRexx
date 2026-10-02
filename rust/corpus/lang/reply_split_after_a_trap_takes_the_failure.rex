/* A trap handled at the REPLY clause's end, then an error in the rest of
   the method: the sender has its value and the error ends the
   continuation, not the sender. */
signal on syntax name caught
say 'got' .k~new~m
call SysSleep 0.2
say 'main end'
exit 0
caught:
  say 'sender caught' condition('D')
  exit 5
::class k
::method m
  call on notready name nr
  reply 'v' || linein('/nonexistent/file/x')
  say 1/0
  return
nr:
  return
