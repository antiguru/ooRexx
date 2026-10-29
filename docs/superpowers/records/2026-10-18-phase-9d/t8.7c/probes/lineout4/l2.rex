s = .stream~new('zz_l4.txt')
call try "x = s~lineout('a', 'b', 'c', 'd')"
call try "x = s~lineout('a', 1, 'c', 'd')"
call try "x = s~lineout('a', 1, 'c')"
call try "x = s~charout('a', 'b', 'c')"
call try "x = s~charout('a', 1, 'c')"
call try "x = s~linein('b', 1, 'c')"
call try "x = s~linein(1, 1, 'c')"
call try "x = s~charin(1, 1, 'c')"
call try "x = s~lines('N', 'c')"
call try "x = s~chars('c')"
call try "x = s~seek('=1', 'c')"
call try "x = s~open('read', 'c')"
call try "x = s~close('c')"
call try "x = s~query('exists', 'c')"
call try "x = s~qualify('c')"
call try "x = s~description('c')"
call try "x = s~state('c')"
call try "x = s~say('a', 'b')"
call try "x = s~arrayout(.array~of(1), 'b')"
call try "x = s~position('=1', 'c')"
call try "x = s~command('query exists', 'c')"
call try "x = s~uninit('c')"
call try "x = s~flush('c')"
call try "x = s~makearray('line', 'c')"
call try "x = s~supplier('c')"
call try "x = s~arrayin('line', 'c')"
call try "x = s~linein(, , )"
call try "x = s~lineout(, , , )"
call try "x = s~lineout('a', 'b')"
call try "x = s~lineout('a', -1, 'c')"
call try "x = s~lineout('a', 0, 'c')"
x = s~close
exit
try:
  signal on syntax
  interpret arg(1)
  say arg(1) 'ok' x
  return
syntax:
  say arg(1) condition('o')~code '|' condition('o')~message
  return
