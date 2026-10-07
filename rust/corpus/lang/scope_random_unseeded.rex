/* Each top-level activation seeds its RANDOM stream from the activity's
   generator, so two calls of one method draw different values. */
o = .t~new
a = o~m
b = o~m
say 'differ' (a \= b)
say 'main' (random(1,1000000) \= random(1,1000000))
::class t
::method m
  return random(1,1000000)
