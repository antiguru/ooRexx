call t "SysGetMessage(5)"
call t "SysGetMessage(0)"
call t "SysGetMessage(-1)"
call t "SysGetMessage('x')"
call t "SysGetMessage(1.5)"
call t "SysGetMessage(4294967301)"
call t "SysGetMessage(999999999999999999)"
call t "SysGetMessage(1000000000000000000)"
call t "SysGetMessage(5, 'rexx.cat')"
call t "SysGetMessage(5, 'REXX.CAT')"
call t "SysGetMessage(5, '')"
call t "SysGetMessage(5, 'rexx.cat'||'00'x||'zz')"
call t "SysGetMessage(5, .array~new)"
call t "SysGetMessageX(1, 5)"
call t "SysGetMessageX(0, 5)"
call t "SysGetMessageX(2, 5)"
call t "SysGetMessageX(4294967297, 5)"
call t "SysGetMessageX(1)"
call t "SysGetMessageX()"
call t "SysGetMessage(678, , 1, 2, 3, 4, 5, 6, 7, 8, 9)"
call t "SysGetMessage(678, , 1, 2, 3, 4, 5, 6, 7, 8, 9, 10)"
call t "SysGetMessage(678, , 1, 2, 3, 4, 5, 6, 7, 8, 9, )"
call t "SysGetMessage(678, 'nope.cat', 1, 2, 3, 4, 5, 6, 7, 8, 9, 10)"
call t "SysGetMessage(678, , .array~of(1,2), 'a'||'00'x||'b')"
call t "SysGetErrorText(2)"
call t "SysGetErrorText(0)"
call t "SysGetErrorText(-1)"
call t "SysGetErrorText(200)"
call t "SysGetErrorText(133)"
call t "SysGetErrorText(134)"
call t "SysGetErrorText(2147483647)"
call t "SysGetErrorText(-2147483648)"
call t "SysGetErrorText(2147483648)"
call t "SysGetErrorText('x')"
call t "SysGetErrorText()"
call t "SysGetErrorText(1, 2)"
call t "SysGetErrorText(' 2 ')"
call t "SysGetErrorText(2.0)"
call t "SysFormatMessage('&1', .array~new(9))"
call t "SysFormatMessage('&1', .array~new(10))"
call t "SysFormatMessage('&1-&2', .array~new(10)~~put('a', 1))"
call t "SysFormatMessage('&1', .array~new(2,2))"
call t "SysFormatMessage('&1', 'abc')"
call t "SysFormatMessage('&1&2', 'abc'||'0a'x||'def')"
call t "SysFormatMessage('&1', .object~new)"
call t "SysFormatMessage('&1', .nil)"
call t "SysFormatMessage('&1', 12)"
call t "SysFormatMessage('a'||'00'x||'&1', 'q')"
call t "SysFormatMessage(.array~of(1), 'q')"
call t "SysFormatMessage('&1&', 'q')"
call t "SysFormatMessage('&&', 'q')"
call t "SysFormatMessage('&1', 'q', 'r')"
call t "SysFormatMessage(, 'q')"
call t "SysFormatMessage('&1')"
call t "SysFormatMessage('&1&2', .array~of(.array~of(1,2), .object~new))"
a = .array~new; do 5; a~append(1); end
call t "SysFormatMessage('&1', a)"
b = .array~new(20); b[3] = 'x'
call t "SysFormatMessage('&3', b)"
c = .array~of(1,2,3,4,5,6,7,8,9,10); c~delete(10)
call t "SysFormatMessage('&9', c)"
d = .array~of(1,2,3,4,5,6,7,8,9,10); d~remove(10)
call t "SysFormatMessage('&9', d)"
say SysGetErrorText(5)~length
exit
t:
  parse arg expr
  signal on syntax name err
  interpret 'r =' expr
  say expr '->' '['r']'
  return
err:
  say expr '-> syntax' condition('o')~code ':' condition('o')~message
  return
