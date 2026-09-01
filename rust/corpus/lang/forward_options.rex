/* FORWARD's value-carrying options, and the defaults it fills in
   for whatever is not written. Each send below leaves exactly one option
   unnamed, so the default it takes is the thing the line reports: `to` alone
   keeps the message name and the arguments, `message` alone keeps the
   receiver and the arguments, and `arguments`/`array` keep both while
   replacing what the callee sees. A bare FORWARD, which would take every
   default at once, is the shape corpus/oracle-crashes.txt names and is not
   here. */
t = .Target~new
o = .K~new
say 'to' o~m(t)
say 'message' o~renamed(7)
say 'arguments' o~replaced(9)
say 'trimmed' o~trimmed(9)
say 'array' o~arrayed(9)
say 'omitted' o~omitted

::CLASS Target
::METHOD m
  return 'target-m' arg()

::CLASS K
::METHOD m
  use arg t
  forward to (t)
::METHOD renamed
  forward message('SEEN')
::METHOD replaced
  forward message('SEEN') arguments ((1,2))
::METHOD trimmed
  forward message('SEEN') arguments ((1,,3,,))
::METHOD arrayed
  forward message('SEEN') array(4,5)
::METHOD omitted
  forward message('SEEN') array(6,,8)
::METHOD seen
  return 'seen' arg() '['arg(1)']' '['arg(2)']' '['arg(3)']'
