/* CHARIN past the end. A read that gets some characters but not all answers
   them and leaves NOTREADY:EOF, a read that gets none leaves ERROR:0, and a
   start past the end is NOTREADY:EOF before anything is read and leaves the
   read position where it was. A read that succeeds makes the stream READY
   again. Each raise carries what the call answers as RESULT. */
call charout 'f.txt', '1234567890'
call charout 'f.txt'
s = .Stream~new('f.txt')
call on notready name caught
call t 's~charin(9, 6)'
call t 's~charin(, 2)'
call t 's~charin(30)'
call t 's~charin(11)'
call t 's~charin(5)'
call t 's~charin(1, 0)'
call t 's~charin(10, 1)'
call t 's~charin(11, 1)'
call t 's~charin(4, 3)'
s~close
call t "charin('f.txt', 8, 5)"
call t "stream('f.txt', 'S')"
call t "charin('f.txt', 1, 2)"
call t "stream('f.txt', 'S')"
call lineout 'g.txt', 'one'
call lineout 'g.txt'
g = .Stream~new('g.txt')
call t 'g~linein', g
call t 'g~linein', g
call t 'g~linein(1)', g
call off notready
signal on notready name eof
say s~charin(8, 9)
say 'not reached'
eof:
say 'signal' condition('D') condition('O')~result s~description
exit

t:
  hits = ''
  interpret 'r =' arg(1)
  if arg(2, 'E') then shown = arg(2)
  else shown = s
  say arg(1) '[' || r || ']' shown~description hits
  return

caught:
  hits = 'raised' condition('D') '[' || condition('O')~result || ']'
  return
