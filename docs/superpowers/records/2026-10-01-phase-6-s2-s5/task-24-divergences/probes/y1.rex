m0 = .message~new('a', 'length')
m0~notify(.outer~new)
m0~send
say 'main end'
::class outer inherit MessageNotification
::method messageComplete
  m = .message~new(.t~new, 'boom')
  m~notify(.bad~new)
  m~start
  m~wait
  say 'outer waited' m~hasError m~errorCondition~code
::class t
::method boom
  return 1/0
::class bad inherit MessageNotification
::method messageComplete
  use arg msg
  say 'notifier sees' msg~errorCondition~code
  return .nil~foo
