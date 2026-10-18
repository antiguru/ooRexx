names = .array~of('/', '*', 'non-existing-file', 'bad'||'80'x||'name', '/nonexist/x')
ops = .array~of('chars', 'lines', 'linesC', 'linein', 'charin', 'lineout', 'charout', 'lineoutclose')
call on notready name h
do n over names
  do op over ops
    g.0 = 0
    select
      when op = 'chars' then r = chars(n)
      when op = 'lines' then r = lines(n)
      when op = 'linesC' then r = lines(n, 'C')
      when op = 'linein' then r = linein(n)
      when op = 'charin' then r = charin(n)
      when op = 'lineout' then r = lineout(n, 'x')
      when op = 'charout' then r = charout(n, 'x')
      when op = 'lineoutclose' then r = lineout(n)
    end
    say c2x(n) op 'r=['r'] hits='g.0 'S='stream(n) 'D='stream(n,'D') g.1
    call lineout n
  end
end
exit
h:
  g.0 = g.0 + 1
  o = condition('O')
  g.1 = 'desc=['condition('D')'] add=['o~additional~string'] addcls=['o~additional~class~id'] res=['o~result']'
  return
