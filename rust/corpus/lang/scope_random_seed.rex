/* RANDOM's seed belongs to the top-level activation: a method's and a
   routine's seeds never reach main's stream. */
say random(1,1000,7)
say .t~new~m
say random()
call s
say random()
::class t
::method m
  return random(1,1000,11)
::routine s
  call random 1,1000,13
