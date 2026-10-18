m = .message~new('abc','length')
m~notify(.bad~new)
m~start
m~wait
say 'main' m~result
call SysSleep 0.2
say 'end'
::class bad inherit MessageNotification
::method messageComplete
  say 'in notifier'
  return 1/0
