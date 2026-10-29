call try "call lineout 'zz_l4.txt', 'a', 1, 2"
call try "call charout 'zz_l4.txt', 'a', 1, 2"
call try "call linein 'zz_l4.txt', 1, 1, 2"
call try "call charin 'zz_l4.txt', 1, 1, 2"
call try "call lines 'zz_l4.txt', 'N', 2"
call try "call chars 'zz_l4.txt', 2"
call try "call stream 'zz_l4.txt', 'C', 'QUERY EXISTS', 2"
call try "call lineout 'zz_l4.txt', 'a', 1, 2, 3"
call try "call lineout , , , 1"
exit
try:
  signal on syntax
  interpret arg(1)
  say arg(1) 'ok' result
  return
syntax:
  say arg(1) condition('o')~code '|' condition('o')~message
  return
