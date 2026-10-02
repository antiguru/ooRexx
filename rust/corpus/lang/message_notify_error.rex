/* A send that fails notifies with its condition already on the message:
   trapped by the sender, and failing in a started activity. */
m = .message~new(.t~new, 'boom')
m~notify(.n~new)
signal on syntax
m~send
say 'not reached'
syntax:
  say 'trapped' condition('o')~code
r = .rec~new
m2 = .message~new(.t~new, 'boom')
w = .message~new(r, 'note', 'i', m2)
m2~notify(w)
m2~start
w~wait
say 'started' r~seen
::class t
::method boom
  return 1/0
::class n inherit MessageNotification
::method messageComplete
  use arg msg
  say 'notified' msg~hasError msg~completed msg~errorCondition~code
::class rec
::attribute seen
::method note
  expose seen
  use arg msg
  seen = msg~hasError msg~completed msg~errorCondition~code
