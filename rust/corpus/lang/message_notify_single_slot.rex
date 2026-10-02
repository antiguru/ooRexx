/* A failing send tells only the message in the activation's single slot:
   the inner message a held send made took it over, and a started message's
   own send has no activation to hold it. */
m = .message~new(.t~new, 'boom')
m2 = .message~new(m, 'send')
m~notify(.n~new('n1'))
m2~notify(.n~new('n2'))
signal on syntax name s
m2~send
s:
  say m~hasError m2~hasError m2~completed
m2~notify(.n~new('n3'))
m~notify(.n~new('n4'))
m3 = .message~new(.t~new, 'boom')
m4 = .message~new(m3, 'send')
m3~notify(.n~new('n5'))
w = .message~new(.n~new('n6'), 'messageComplete', 'i', m4)
m4~notify(w)
m4~start
w~wait
say 'started' m3~hasError m3~completed m4~hasError m4~completed
::class t
::method boom
  return 1/0
::class n inherit MessageNotification
::method init
  expose tag
  use arg tag
::method messageComplete
  expose tag
  use arg msg
  say 'notified' tag msg~hasError msg~completed
