parse source . . me
d = me~left(me~lastPos('/'))
call v 'r', 'both binary reclength 4', 'call lineout f'
call v 'p', 'both binary reclength 4', 'call lineout f, , 4; call lineout f'
call v 'n', 'both binary', 'call lineout f'
exit
v:
  f = d || arg(1) || '.dat'
  call stream f, 'c', 'open write replace'
  call charout f, 'AAAABBBBCCCCDDDD'
  call charout f
  call stream f, 'c', 'open' arg(2)
  call charout f, 'x', 5
  interpret arg(3)
  say arg(1) translate(charin(f, 1, chars(f)), '_', ' ')
  call stream f, 'c', 'close'
  return
