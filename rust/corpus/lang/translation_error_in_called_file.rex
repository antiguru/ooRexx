/* A translation error in a called file is reported against that file,
   from the frame translating it: a duplicate ::ROUTINE, and a library the
   translation fails to load before a later clause's error. */
call write 'duprt.rex', 'say "duprt prolog"', '::routine r', '::routine r'
call write 'badlib.rex', 'say "badlib prolog"', -
  '::routine r external "LIBRARY nosuchlibzz"', '::routine s', 'say ('
signal on syntax name c1
call duprt.rex
c1: call show
signal on syntax name c2
call badlib.rex
c2: call show
call duprt.rex
exit

write: procedure
  parse arg file
  call stream file, 'c', 'open write replace'
  do i = 2 to arg()
    call lineout file, arg(i)
  end
  call lineout file
  return

show:
  o = condition('o')
  say o~code o~position
  do f over o~stackframes
    say f~type filespec('name', f~name) f~line
  end
  return
