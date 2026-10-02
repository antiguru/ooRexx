/* A send that fails untrapped notifies before the error is reported. */
m = .message~new(.t~new, 'boom')
m~notify(.n~new)
m~send
::class t
::method boom
  return 1/0
::class n inherit MessageNotification
::method messageComplete
  use arg msg
  say 'notified' msg~hasError msg~completed msg~errorCondition~code
