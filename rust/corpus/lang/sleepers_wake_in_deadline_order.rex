-- Two started activities sleep at once, the longer one started first: they
-- wake in deadline order, and the run takes about the longer sleep, well
-- below the sum of both.
call time 'R'
a = .t~new~start('nap', 'A', 2)
b = .t~new~start('nap', 'B', 1)
say 'main waits'
a~wait
b~wait
say 'below the sum' (time('E') < 2.8)
::class t
::method nap
  use arg name, secs
  call SysSleep secs
  say name 'woke'
