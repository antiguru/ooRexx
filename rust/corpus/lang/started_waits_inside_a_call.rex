/* The wait is inside a method the started method sends to, so the started
   activity parks with a level of its own below the waiting one. */
m = .message~new('abc', 'length')
t = .w~new~start('OUTER', m)
m~send
say t~result
::class w
::method outer
  use arg m
  return self~inner(m) + 1
::method inner
  use arg m
  say 'inner waits'
  r = m~result
  say 'inner got' r
  return r
