/* CHARS and LINES on a FIFO take nothing: a second stream object on it, and
   the same object after a close and reopen, read every byte. */
address system 'mkfifo ff'
f = .stream~new('ff')
say 'open' f~open('both nobuffer')
zz = f~charout('abcd' || '0a'x || 'ef' || '0a'x)
say 'counts' f~chars f~lines f~lines('N')
g = .stream~new('ff')
say 'open2' g~open('read nobuffer')
say 'second object reads [' || c2x(g~charin(, 2)) || ']'
say 'counts' f~chars f~lines
say 'close' f~close 'reopen' f~open('both nobuffer')
say 'after reopen [' || c2x(f~charin(, 2)) || ']'
say 'rest [' || f~linein || ']'
