/* A condition trapped before a REPLY and propagated after it is reported on
 * the method's new activity with the sender's traceback line, which the
 * condition object kept. */
o = .t~new
say 'got' o~m
call SysSleep 0.2
say 'main done'
::class t
::method m
  signal on syntax
  x = 1/0
  return 0
syntax:
  reply 1
  call s
  return
s:
  say 'cont propagates'
  raise propagate
