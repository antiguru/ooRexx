call try 1, 'call SysFileTree "", f.'
call try 2, 'call SysFileTree "%", ""'
call try 3, 'call SysFileTree "%", "s.1"'
call try 4, 'f.0 = "keep"; call SysFileTree "%", f., ""'
call try 5, 'call SysFileTree "%", f., "X"'
call try 6, 'call SysFileTree "%", f., "f d"'
call try 7, 'call SysFileTree "%", f., , " "'
call try 8, 'call SysFileTree "%", f., , "******"'
call try 9, 'call SysFileTree "%", f., , , "x"'
call try 10, 'call SysFileTree "", "", "X"'
call try 11, 'call SysFileTree "", f., "X"'
call try 12, 'call SysFileTree "%", f., "X", "x"'
call try 13, 'a = .array~of(1,2); call SysFileTree "", a; say "a items" a~items'
call try 14, 'call SysFileTree "%", f., "", "x"'
call try 15, 'call SysFileTree "%", f., copies("Q", 300)'
call try 16, 'call SysFileTree "%", f., , copies("+", 300)'
call try 17, 'call SysFileTree "%", f., "fo"||"00"x||"Z"'
call try 18, 'call SysFileTree "%", .stem~new'
call try 19, 'call SysFileTree "%", f., , "+-*+-"'
call try 20, 'call SysFileTree "%", f., , "", ""'
call try 21, 'call SysFileTree "%", f., .nil'
call try 22, 'call SysFileTree "%", f., , "+"||"00"x||"x"'
call try 23, 'call SysFileTree "%", f., , , , ""'
call try 24, 'call SysFileTree'
call try 25, 'call SysFileTree "%"'
call try 26, 'call SysFileTree "%", "name"; say "name.0" name.0'
call try 27, 'call SysFileTree .nil, f.'
call try 28, 'call SysFileTree "%", .nil'
call try 29, 'call SysFileTree "%", 12'
call try 30, 'call SysFileTree "%", f., "d"||"ff"x'
call try 31, 'call SysFileTree "%", f., "%%"'
call try 32, 'call SysFileTree "a"||"00"x, f.'
call try 33, 'call SysFileTree "00"x, f.'
exit
try: procedure
  parse arg n, code
  signal on syntax
  f.0 = 'unset'
  interpret code
  say n 'returned' result 'f.0' f.0
  return
syntax:
  o = condition('o')
  say n 'SYNTAX' o~code '|' o~message '| f.0' f.0
  return
