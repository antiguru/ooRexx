signal on notready name nr
call v 'b1', 'BOTH BINARY RECLENGTH 4', 'call lineout f'
call v 'b2', 'BOTH BINARY RECLENGTH 4', 'call lineout f, "y"'
call v 'b3', 'BOTH BINARY RECLENGTH 4', 'call stream f, "C", "CLOSE"'
call v 'b4', 'BOTH BINARY RECLENGTH 4', 'nop'
call v 'b5', 'BOTH BINARY RECLENGTH 4', 'call charout f'
call v 'b6', 'BOTH BINARY RECLENGTH 4', 'call lineout f, , 4; call lineout f'
call v 'b7', 'BOTH BINARY RECLENGTH 4', 'call lineout f', 'xxxx'
call v 'b8', 'BOTH BINARY RECLENGTH 4', 'call stream f, "C", "CLOSE"', 'xxxx'
call v 'w1', 'WRITE BINARY RECLENGTH 4', 'call lineout f'
call v 'w2', 'WRITE BINARY RECLENGTH 4', 'call lineout f, "y"'
call v 'n1', 'BOTH BINARY', 'call lineout f'
call v 'n2', 'BOTH BINARY', 'call lineout f, "y"'
call v 'n3', 'BOTH BINARY', 'call stream f, "C", "CLOSE"'
call v 't1', 'BOTH', 'call lineout f'
call v 't2', 'BOTH', 'call lineout f, "y"'
exit
v:
  f = arg(1) || '.dat'
  call charout f, 'AAAABBBBCCCCDDDD'
  call charout f
  call stream f, 'c', 'open' arg(2)
  w = 'x'
  if arg(4, 'E') then w = arg(4)
  call charout f, w, 5
  interpret arg(3)
  call stream f, 'c', 'close'
  say arg(1) translate(charin(f, 1, chars(f)), '_|', ' ' || '0a'x)
  call stream f, 'c', 'close'
  return
nr: say 'NOTREADY' condition('D'); exit
