/* An error installing a called file is reported against that file, and one
   its translation raises comes from the frame translating it. */
call write 'nolib.rex', 'say "nolib prolog"', -
  '::routine r external "LIBRARY nosuchlibzz"'
call write 'cyc.rex', 'say "cyc prolog"', '::class a subclass b', -
  '::class b subclass a'
call write 'nosuper.rex', 'say "nosuper prolog"', '::class t subclass nosuchzz'
signal on syntax name c1
call nolib.rex
c1: call show
signal on syntax name c2
call cyc.rex
c2: call show
signal on syntax name c3
call nosuper.rex
c3: call show
p = .Package~new('pk', .array~of('::class a subclass b', '::class b subclass a'))
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
  say o~code o~position changestr(directory() || '/', o~message, '')
  do f over o~stackframes
    say f~type filespec('name', f~name) f~line
  end
  return
