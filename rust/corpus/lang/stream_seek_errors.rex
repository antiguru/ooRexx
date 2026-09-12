/* Every SEEK and QUERY POSITION option shape that raises. Both tables check
   mutual exclusion before anything moves -- a repeated token counts as a
   clash with itself -- so every rejected option leaves the position alone,
   which the reading between the two groups shows. SYS is a QUERY-only token
   and SEEK rejects it. */
say 'seek errors'
seed = .Stream~new('f.txt')
zz = seed~lineout('alpha')
zz = seed~lineout('beta')
zz = seed~lineout('gamma')
zz = seed~close
s = .Stream~new('f.txt')
say 'open' s~open('read')
bad = .array~of('write', 'read', '', 'char', 'line', 'read write', 'write read',,
      'char line', 'line char', 'read read', 'write write', 'char char',,
      '= 5 <', '=1 =2', '+ - 3', 'sys', 'sys 1', 'bogus', '1.5', '1e1', '-1')
do i = 1 to bad~items
  say right(i, 2) 'seek  [' || bad[i] || '] [' || trySeek(s, bad[i]) || ']'
end
say 'position after [' || s~query('position') || ']'
q = .array~of('read write', 'write read', 'sys char', 'sys line', 'sys read',,
      'sys write', 'sys sys', 'char line', 'read read', 'char char', 'bogus',,
      'read line', 'write line', 'read char')
do i = 1 to q~items
  say right(i, 2) 'query [' || q[i] || '] [' || tryQuery(s, q[i]) || ']'
end
say 'close' s~close
exit 0

trySeek: procedure
  use arg obj, o
  signal on syntax name seekfailed
  return obj~seek(o)
seekfailed:
  return 'RAISED'

tryQuery: procedure
  use arg obj, o
  signal on syntax name queryfailed
  return obj~query('position' o)
queryfailed:
  return 'RAISED'
