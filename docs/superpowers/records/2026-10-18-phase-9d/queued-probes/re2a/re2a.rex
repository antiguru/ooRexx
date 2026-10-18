m = .message~new(.t~new, 'boom')
m2 = .message~new(m, 'send')
signal on syntax name s
m2~send
s: say m~hasError m2~hasError m2~completed
::class t
::method boom
  return 1/0
