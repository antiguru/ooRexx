m0 = .message~new('a', 'length')
m0~notify(.outer~new)
m0~send
say 'main end'
::class outer inherit MessageNotification
::method messageComplete
  m = .message~new('abc', 'length')
  m~notify(.bad~new)
  m~start
  m~wait
  say 'outer waited' m~result
::class bad inherit MessageNotification
::method messageComplete
  say 'in notifier'
  return 1/0
