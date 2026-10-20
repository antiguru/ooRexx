call mk 'l1', 'aaa'||'0a'x||'bbb'||'0a'x||'ccc'||'0a'x
f='l1'
say 'append' stream(f,'c','open both append nobuffer')
say 'co@1' charout(f,'XY',1) 'w' stream(f,'c','query position write')
say 'lo@1' lineout(f,'Z',1) 'w' stream(f,'c','query position write')
say 'seek' stream(f,'c','seek =2 write')
say 'co' charout(f,'Q')
call stream f,'c','close'
say show(f)
call mk 'w1', 'aaa'||'0a'x||'bbb'||'0a'x
f='w1'
say 'write' stream(f,'c','open write nobuffer')
say 'ci' charin(f,1,3) stream(f,'D')
say 'li' linein(f,1) stream(f,'D')
say 'chars' chars(f) 'lines' lines(f) lines(f,'C')
say 'seekline' stream(f,'c','seek =2 read line') stream(f,'D')
say 'qwl' stream(f,'c','query position write line')
call stream f,'c','close'
call mk 'q1', 'aaa'||'0a'x||'bbb'||'0a'x||'ccc'||'0a'x
f='q1'
say stream(f,'c','open both nobuffer')
say 'qwl-end' stream(f,'c','query position write line') 'qrl' stream(f,'c','query position read line')
call charout f,,1
say 'qwl@1' stream(f,'c','query position write line')
call charout f,,5
say 'qwl@5' stream(f,'c','query position write line')
call charout f,,4
say 'qwl@4' stream(f,'c','query position write line')
call charin f,4,0
say 'qrl@4' stream(f,'c','query position read line')
call charin f,5,0
say 'qrl@5' stream(f,'c','query position read line')
call charin f,1,2
say 'lc after charin' lines(f,'C') lines(f,'C') lines(f,'C')
say 'li' linein(f) 'lc' lines(f,'C') lines(f,'C')
call stream f,'c','close'
call mk 'e1', 'aaa'||'0a'x||'bbb'||'0a'x
f='e1'
say stream(f,'c','open both nobuffer')
say 'lo@5' lineout(f,'Z',5) 'w' stream(f,'c','query position write') stream(f,'D')
say 'co@20' charout(f,'Y',20) stream(f,'D')
say 'ci@30' '['charin(f,30,2)']' stream(f,'D')
say 'ci@9' '['charin(f,9,2)']' stream(f,'D')
call stream f,'c','close'
say show(f)
exit
mk: procedure
  parse arg n, d
  call stream n,'c','open write replace'; call charout n,d; call stream n,'c','close'
  return
show: procedure
  parse arg n
  s = charin(n,1,chars(n)); call stream n,'c','close'
  return '['s~changestr('0a'x,'|')~changestr('00'x,'@')']'
