/* Every argument shape the stream builtins refuse, and the ones they accept
   that look as though they should not. STREAM's arity depends on its own
   second argument -- S and D refuse a third, C requires one -- so those two
   are checked in the builtin rather than by the shared arity table. An empty
   name is refused by STREAM alone; every other reader takes it as the default
   input or output, and a blank name is a name. */
say 'stream builtin errors'
seed = .Stream~new('e.txt')
zz = seed~lineout('one')
zz = seed~close
bad = .array~of('stream()', "stream('')", "stream('e.txt','X')",,
      "stream('e.txt','')", "stream('e.txt','C')", "stream('e.txt','S','open')",,
      "stream('e.txt','D','open')", "stream('e.txt','C','bogus')",,
      "lines('e.txt','X')", "lines('e.txt','')", "lines('e.txt','C','extra')",,
      "chars('e.txt','x')", "qualify()", "qualify('a','b')")
do i = 1 to bad~items
  say right(i, 2) '[' || bad[i] || '] [' || try(bad[i]) || ']'
end
/* The shapes that answer rather than raise. */
say 'blank name  [' || stream(' ') || ']'
say 'S abbrev    [' || stream('e.txt','s') || ']'
say 'C abbrev    [' || stream('e.txt','c','query exists') \== '' || ']'
say 'nosuch S    [' || stream('nosuch.txt','S') || ']'
say 'nosuch D    [' || stream('nosuch.txt','D') || ']'
say 'lines count [' || lines('e.txt','count') || ']'
say 'lines norm  [' || lines('e.txt','normal') || ']'
exit 0

try: procedure
  use arg expression
  signal on syntax name failed
  interpret 'answer =' expression
  return answer
failed:
  return 'RAISED'
