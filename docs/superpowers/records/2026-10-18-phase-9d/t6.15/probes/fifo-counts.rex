f = 'ff'
address system 'mkfifo ff'
say stream(f,'c','open both nobuffer')
say charout(f,'abcd') lineout(f,'e')
say 'ch' chars(f) 'ln' lines(f) 'lc' lines(f,'C')
say 'ci' charin(f,,2) 'li' linein(f)
