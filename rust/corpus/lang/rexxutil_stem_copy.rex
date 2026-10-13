/* SysStemCopy: overlay and insert, the from, to and count arguments, the stem
   shapes that are not plain arrays on either side, a stem copied onto itself,
   and every error the routine raises. */
abc = "a.0 = 3; a.1 = 'a'; a.2 = 'b'; a.3 = 'c'"
def = "b.0 = 3; b.1 = 'd'; b.2 = 'e'; b.3 = 'f'"
call t abc"; "def, "call SysStemCopy 'a.', 'b.'"
call t abc"; "def, "call SysStemCopy a., b., , 2"
call t abc"; "def, "call SysStemCopy 'a.', 'b.', , 4"
call t abc"; "def, "call SysStemCopy 'a.', 'b.', 2, 2, 1"
call t abc"; "def, "call SysStemCopy 'a.', 'b.', 2, , 0"
call t abc"; "def, "call SysStemCopy 'a.', 'b.', 3"
call t abc"; "def, "call SysStemCopy 'a.', 'b.', 5"
call t abc"; "def, "call SysStemCopy 'a.', 'b.', , , , 'Overlay'"
call t abc"; "def, "call SysStemCopy 'a.', 'b.', , , , 'i'"
call t abc"; "def, "call SysStemCopy 'a.', 'b.', , 2, , 'Insert'"
call t abc"; "def, "call SysStemCopy 'a.', 'b.', 2, 4, 2, 'I'"
call t abc"; "def, "call SysStemCopy 'a.', 'b.', 2, 2, 0, 'I'"
call t abc"; "def, "call SysStemCopy 'a.', 'b.', 3, 1, , 'I'"
call t abc"; "def, "call SysStemCopy 'a.', 'b.', 5, 1, , 'I'"
call t abc, "call SysStemCopy 'a.', 'b.'"
call t abc, "call SysStemCopy 'a', 'b', , , , 'I'"
call t abc, "call SysStemCopy 'a.', 'a.', , 2, , 'I'"
call t abc, "call SysStemCopy 'a.', 'a.', 2, 1"
call t abc"; b. = 'z'", "call SysStemCopy 'a.', 'b.', , 1, 2"
call t abc"; b. = 'z'; b.0 = 1", "call SysStemCopy 'a.', 'b.', , 2, , 'I'"
call t "a. = 'd'; a.0 = 2; a.1 = 'q'; a.2 = 'p';"def, "call SysStemCopy 'a.', 'b.'"
call t "a.0 = '2.0'; a.1 = 'q'; a.2 = 'p';"def, "call SysStemCopy 'a.', 'b.', , , , 'I'"
call t "a.0 = ' 2 '; a.1 = 'q'; a.2 = 'p'; b.0 = '1E0'; b.1 = 'r'", "call SysStemCopy 'a.', 'b.', , , , 'I'"
call t "a.0 = 2; a.1 = 'q'; a.2 = 'p'; a.x = 'r'; a.1.5 = 's'; a.3 = 'o'", "call SysStemCopy 'a.', 'b.'"
call t "a.0 = 2; a.1 = .array~of('z', 'y'); a.2 = 'p'", "call SysStemCopy 'a.', 'b.'; say (a.1 == b.1) b.1~class~id"
call t "a.0 = -1;"def, "call SysStemCopy 'a.', 'b.'"
call t "a.0 = 0;"def, "call SysStemCopy 'a.', 'b.'"
call t "a.0 = 1; a.1 = 'q'; b.0 = -1", "call SysStemCopy 'a.', 'b.'"
call t "a.0 = 1; a.1 = 'q'; b.0 = -2", "call SysStemCopy 'a.', 'b.', , , , 'I'"
call t abc"; "def, "r = SysStemCopy('a.', 'b.'); say 'returned' r"
-- errors
call t abc"; "def, "call SysStemCopy 'a.', 'b.', , , , 'Q'"
call t abc"; "def, "call SysStemCopy 'a.', 'b.', , , , ''"
call t abc"; "def, "call SysStemCopy 'a.', '*'"
call t abc"; "def, "call SysStemCopy 'a.b', 'b.'"
call t abc"; "def, "call SysStemCopy 'a.', 'b.', 0"
call t abc"; "def, "call SysStemCopy 'a.', 'b.', , 'x'"
call t abc"; "def, "call SysStemCopy 'a.', 'b.', , , -1"
call t abc"; "def, "call SysStemCopy 'a.', 'b.', , , 'y'"
call t abc"; "def, "call SysStemCopy 'a.', 'b.', 1, 1, 1, 'O', 1"
call t abc"; "def, "call SysStemCopy 'a.'"
call t abc"; "def, "call SysStemCopy , 'b.'"
call t abc"; "def, "call SysStemCopy 'a.', 'b.', , , , .nil"
call t abc"; "def, "call SysStemCopy 'a.', 'b.', , 5"
call t abc"; "def, "call SysStemCopy 'a.', 'b.', 2, , 3"
call t "a.0 = 0;"def, "call SysStemCopy 'a.', 'b.', , , 0"
call t "a. = 'd';"def, "call SysStemCopy 'a.', 'b.'"
call t "a.1 = 'q';"def, "call SysStemCopy 'a.', 'b.'"
call t "a.0 = '2.5'; a.1 = 'q'; a.2 = 'p';"def, "call SysStemCopy 'a.', 'b.'"
call t "a.0 = ''; a.1 = 'q';"def, "call SysStemCopy 'a.', 'b.'"
call t abc"; b.0 = 'x'; b.1 = 'q'", "call SysStemCopy 'a.', 'b.'"
call t abc"; b.0 = 1.5", "call SysStemCopy 'a.', 'b.'"
call t "a. = 'd'; a.0 = 2; a.1 = 'q';"def, "call SysStemCopy 'a.', 'b.'"
call t "a.0 = 2; a.01 = 'q'; a.02 = 'p';"def, "call SysStemCopy 'a.', 'b.'"
call t "a.0 = 3; a.1 = 'q'; a.3 = 'p';"def, "call SysStemCopy 'a.', 'b.', 2"
call t "a.0 = '1E1'; a.1 = 'q'; a.2 = 'p';"def, "call SysStemCopy 'a.', 'b.'"
call t abc"; "def"; drop b.2", "call SysStemCopy 'a.', 'b.', , , , 'I'"
call t abc"; b. = 'z'; b.0 = 2; b.1 = 'y'", "call SysStemCopy 'a.', 'b.', , 1, , 'I'"
call t "a.0 = 1234567890; a.1 = 'q';"def, "call SysStemCopy 'a.', 'b.'"
a.0 = 1; a.1 = 'q'
call SysStemCopy 'a.', 'b.', 'x'
exit

t: procedure
  parse arg setup, code
  interpret setup
  signal on syntax name bad
  interpret code
  say 'ok   ' code '|' show()
  return
bad:
  c = condition('O')
  say 'error' code '|' c~code c~message '|' show()
  return

show: procedure expose a. b.
  o = ''
  do i = 0 to 4; o = o a.i; end
  o = o '/'
  do i = 0 to 7; o = o b.i; end
  return o
