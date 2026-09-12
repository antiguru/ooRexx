/* CHARS and LINES, and the line count the oracle caches. The count after a
   CHARIN is the interesting one: the cache is stored with the line position
   zeroed, so the second count answers one less than the first, and a reader
   that recounts every time answers differently. */
say 'chars and lines'
c = .Stream~new('lc.txt')
say 'lineout' c~lineout('a') c~lineout('bb') c~lineout('ccc')
say 'state [' || c~state || ']'
say 'close [' || c~close || ']'
t = .Stream~new('lc.txt')
say 'count' t~lines t~lines t~lines
say 'normal' t~lines('N')
say 'chars' t~chars
say 'linein [' || t~linein || ']'
say 'count after a line read' t~lines t~lines
say 'charin [' || t~charin || ']'
say 'count after a char read' t~lines t~lines
say 'rest of the line [' || t~linein || ']'
say 'left' t~lines t~chars
say 'last line [' || t~linein || ']'
say 'state [' || t~state || '] [' || t~description || ']'
/* Past the end the two reads answer differently, and both are measured:
   LINEIN leaves NOTREADY:EOF, CHARIN leaves a bare ERROR:0. */
say 'eof linein [' || t~linein || ']'
say 'state [' || t~state || '] [' || t~description || ']'
say 'eof charin [' || t~charin || ']'
say 'state [' || t~state || '] [' || t~description || ']'
say 'close [' || t~close || ']'
/* A name nothing resolves to answers 0 and leaves ERROR:2, and reading it
   creates nothing. */
n = .Stream~new('nosuch.txt')
say 'missing lines' n~lines
say 'state [' || n~state || '] [' || n~description || ']'
say 'missing chars' n~chars
say 'missing exists [' || n~query('exists') || ']'
