/* Read and write positions on a NOBUFFER stream, where the oracle keeps the
   documented positions. A write starts at the write position and moves it by
   what it wrote, whether a line number placed it or not, and the next write
   follows it. A read, CHARS or LINES leaves the write position where
   it was. A line SEEK on the write side places the next write. Binary record
   streams place each write at the record it names. */
call scenario 'line then lineout', 'L2 new', 'L tail'
call scenario 'line then charout', 'L2 new', 'C tail'
call scenario 'line then charout twice', 'L2 new', 'C ab', 'C cd'
call scenario 'charout at then lineout', 'C7 XY', 'L tail'
call scenario 'linein, line, lineout', 'R', 'L3 new', 'L tail'
call scenario 'line, linein, lineout', 'L2 new', 'R', 'L tail'
call scenario 'line, seek, lineout', 'L2 new', 'S =4 write', 'L tail'
call scenario 'seek write line, lineout', 'S =2 write line', 'L tail'
call scenario 'seek line, lineout', 'S =3 line', 'L tail'
call scenario 'line twice', 'L2 new', 'L3 more'
call scenario 'line, chars, lineout', 'L2 new', 'N', 'L tail'
call scenario 'line, lines, lineout', 'L2 new', 'M', 'L tail'
call scenario 'seek from end line, charout at', 'S <2 read line', 'C9 RR'
call binary 'record line then lineout', 'L2 new', 'L tail'
call binary 'linein, record line, lineout', 'R', 'L1 ab', 'L cd'
exit

scenario:
  f = 'f' || arg(1)~changestr(' ', '_')~changestr(',', '') || '.txt'
  call charout f, 'line1' || '0a'x || 'line2' || '0a'x || 'line3' || '0a'x
  call charout f
  call stream f, 'c', 'open both nobuffer'
  call run arg(1), arg(2), arg(3), arg(4)
  return

binary:
  f = 'b' || arg(1)~changestr(' ', '_')~changestr(',', '') || '.bin'
  call charout f, 'AAAABBBBCCCCDDDD'
  call charout f
  call stream f, 'c', 'open both binary reclength 4 nobuffer'
  call run arg(1), arg(2), arg(3), arg(4)
  return

run:
  out = arg(1) ':'
  do i = 2 to arg()
    if arg(i) == '' then iterate
    parse value arg(i) with op text
    select
      when op = 'L' then r = lineout(f, text)
      when op~left(1) = 'L' then r = lineout(f, text, op~substr(2))
      when op = 'C' then r = charout(f, text)
      when op~left(1) = 'C' then r = charout(f, text, op~substr(2))
      when op = 'R' then r = linein(f)
      when op = 'I' then r = charin(f, , 2)
      when op = 'N' then r = chars(f)
      when op = 'M' then r = lines(f)
      when op = 'S' then r = stream(f, 'c', 'seek' text)
    end
    out = out op'='r 'w' || stream(f, 'c', 'query position write') ,
      'r' || stream(f, 'c', 'query position read')
  end
  call stream f, 'c', 'close'
  say out
  say '   [' || charin(f, 1, 60)~changestr('00'x, '@')~changestr('0a'x, '|') || ']'
  call stream f, 'c', 'close'
  return
