o = .w~new
m = o~start('SLOW')
say 'caller: past the start'
do i = 1 to 5
  say 'caller: working' i
end
say 'caller: now asking for the result'
say 'caller: got' m~result
::class w
::method slow
  do i = 1 to 5
    say '  slow: step' i
  end
  return 'done'
