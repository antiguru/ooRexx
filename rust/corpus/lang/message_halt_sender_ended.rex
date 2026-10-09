/* A message sent from an activity that has since ended: Message~halt
   answers 1 and halts nothing. */
m = .message~new(.k~new, 'm')
w = .w~new~start('go', m)
say w~result
say m~halt
say 'main end'
::class w
::method go
  use arg m
  return m~send
::class k
::method m
  return 'v'
