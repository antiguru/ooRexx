/* One line per ruled rule of Phase 9d Task 6.15 fix round 1, each on a
   fresh file "line1|line2|line3|". */
call mk 'a'; call stream 'a', 'c', 'open write nobuffer'
say 'R1 write-only:' charin('a', 1, 3) stream('a', 'S') chars('a') lines('a', 'C') stream('a', 'c', 'seek =2 read line')
call mk 'b'; call stream 'b', 'c', 'open both nobuffer'
call charin 'b', 6, 1
say 'R2 line seek:' stream('b', 'c', 'seek +1 read line') linein('b')
call mk 'c'; call stream 'c', 'c', 'open both nobuffer'
call charin 'c', 8, 1; call stream 'c', 'c', 'query position read line'
say 'R2 linein n:' linein('c', 2)
call mk 'd'; call stream 'd', 'c', 'open both nobuffer'
call charout 'd', , 8; w8 = stream('d', 'c', 'query position write line')
call charin 'd', 1, 18; rend = stream('d', 'c', 'query position read line')
call charin 'd', 7, 0; r7 = stream('d', 'c', 'query position read line')
say 'R3 line query:' w8 rend r7
call mk 'e'; call stream 'e', 'c', 'open both nobuffer'
call charin 'e', 2, 1
say 'R4 lines count:' lines('e', 'C') lines('e', 'C')
call mk 'f'; call stream 'f', 'c', 'open both nobuffer'
say 'invalid seek:' stream('f', 'c', 'seek <99 read') stream('f', 'S') stream('f', 'c', 'query position read')
call mk 'g'; call stream 'g', 'c', 'open read nobuffer'
say 'invalid seek on READ:' stream('g', 'c', 'seek =2 write') stream('g', 'S') stream('g', 'c', 'query position write')
call charout 'z', 'ab' || '1a'x; call charout 'z'
call stream 'z', 'c', 'open both nobuffer'
say 'ctrl-Z:' stream('z', 'c', 'query position write') lineout('z', 'x'); call stream 'z', 'c', 'close'
say '   ' c2x(charin('z', 1, 9)); call stream 'z', 'c', 'close'
say 'devnull:' chars('/dev/null') lines('/dev/null')
n = 0
do while lines('/dev/null') > 0 & n < 3
  n = n + 1
  l = linein('/dev/null')
end
say '   loop ran' n
exit
mk: procedure
  parse arg f
  call stream f, 'c', 'open write replace'
  call charout f, 'line1' || '0a'x || 'line2' || '0a'x || 'line3' || '0a'x
  call stream f, 'c', 'close'
  return
