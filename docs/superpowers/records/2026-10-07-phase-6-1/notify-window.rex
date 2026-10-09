m = .message~new('abc', 'reverse')
m~notify(.slow~new)
m~start
call SysSleep 0.3
w = .message~new(.counter~new, 'bump')
m~notify(w)
say 'notified' m~completed
w~wait
say 'waited'
exit
::class slow inherit MessageNotification
::method messageComplete
  call SysSleep 1
::class counter
::method bump
  return 1
