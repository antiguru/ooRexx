/* The stream builtins as sends. Each resolves its first argument through the
   activation's stream table and sends the message named after it, so a builtin
   and the method it stands for share one position -- and the arguments arrive
   positionally, an omitted middle one still omitted. LINES is the exception
   worth pinning: the builtin defaults to NORMAL where the method defaults to
   COUNT, so lines(f) answers whether a line is there and f~lines answers how
   many are left. */
say 'stream builtins'
seed = .Stream~new('f.txt')
zz = seed~lineout('one')
zz = seed~lineout('two')
zz = seed~lineout('three')
zz = seed~close
say 'linein   [' || linein('f.txt') || ']'
say 'linein   [' || linein('f.txt') || ']'
say 'lines    [' || lines('f.txt') || ']'
say 'lines C  [' || lines('f.txt','C') || ']'
say 'lines N  [' || lines('f.txt','N') || ']'
say 'chars    [' || chars('f.txt') || ']'
say 'charin   [' || charin('f.txt',1,3) || ']'
/* The builtin's name is qualified before the table is consulted, so these
   three spellings are one entry and one position. */
say 'dotslash [' || linein('./f.txt') || ']'
say 'qualified[' || linein(qualify('f.txt')) || ']'
/* An object built with ~new is never in the table, so it reads from its own
   position rather than continuing the builtin's. */
own = .Stream~new('f.txt')
say 'object   [' || own~linein || ']'
say 'close    [' || own~close || ']'
say 'builtin  [' || linein('f.txt') || ']'
/* STREAM's three operations. The second argument is read by its first letter
   only, S is the default, and D answers the description with its colon. */
say 'S        [' || stream('f.txt','S') || ']'
say 'default  [' || stream('f.txt') || ']'
say 'D        [' || stream('f.txt','D') || ']'
say 'C size   [' || stream('f.txt','C','query size') || ']'
say 'C close  [' || stream('f.txt','C','close') || ']'
say 'S after  [' || stream('f.txt','S') || ']'
/* A LINEOUT with neither a string nor a line closes the stream and drops the
   entry, so the next builtin naming it starts again from line one. */
zz = lineout('g.txt','alpha')
zz = lineout('g.txt','beta')
say 'g close  [' || lineout('g.txt') || ']'
say 'g first  [' || linein('g.txt') || ']'
say 'g second [' || linein('g.txt') || ']'
/* CHAROUT writes at a position and answers the residual count. */
say 'charout  [' || charout('g.txt','ZZ',2) || ']'
say 'g close  [' || lineout('g.txt') || ']'
say 'g bytes  [' || linein('g.txt') || ']'
