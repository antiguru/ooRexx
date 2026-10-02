/* Message~notify: each object it named is sent messageComplete when the send
   completes, at once where it already has; a copy has its own list, and a
   Message is itself a notifier, whose messageComplete sends it. */
m = .message~new('abc', 'reverse')
say m~isA(.MessageNotification)
n1 = .counter~new
mn1 = .message~new(n1, 'bump')
m~notify(mn1)
say m~send n1~count mn1~completed
n2 = .counter~new
m~notify(n2)
say 'at once' n2~count (n2~last == m)
m~send
say n1~count n2~count
m2 = m~copy
n3 = .counter~new
m2~notify(n3)
say 'copy' n3~count m2~completed m2~hasResult
m~send
m2~send
say n1~count n2~count n3~count (n2~last == m2) (n3~last == m2)
r = m~reply
w = .message~new(.counter~new, 'bump')
r~notify(w)
w~wait
say 'reply' n1~count n2~count (n2~last == r)
m~start
w = .message~new(.counter~new, 'bump')
m~notify(w)
w~wait
say 'start' n1~count n2~count (n2~last == m)
t = .message~new('xyz', 'length')
t~triggered('ignored')
say 'triggered' t~result
exit
::class counter inherit MessageNotification
::attribute count
::attribute last
::method init
  expose count last
  count = 0
  last = .nil
::method bump
  expose count
  count += 1
::method messageComplete
  expose last
  use strict arg message
  self~bump
  last = message
