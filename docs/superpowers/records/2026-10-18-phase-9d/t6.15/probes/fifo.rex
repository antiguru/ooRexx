f = 'ff'
address system 'mkfifo ff'
say stream(f,'c','open both' 'nobuffer')
say 'type' stream(f,'c','query streamtype')
say 'co' charout(f,'abc') stream(f,'S')
say 'lo' lineout(f,'de') stream(f,'S')
say 'chars' chars(f) 'lines' lines(f) lines(f,'C')
say 'qp' stream(f,'c','query position read') stream(f,'c','query position write')
say 'ci' charin(f,,2) stream(f,'S')
say 'li' linein(f) stream(f,'S')
signal on syntax
say 'seek' stream(f,'c','seek =1')
exit
syntax: say 'S' condition('O')~code
