/* A started method waits on ~result of a message main creates before the
   start and sends only after it, and main waits on the started one. */
m = .message~new('abc', 'length')
t = .w~new~start('WAITON', m)
m~send
say t~result
::class w
::method waiton
  use arg m
  return m~result
