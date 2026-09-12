/* QUERY STREAMTYPE answers from the descriptor rather than from the name:
   UNKNOWN until something opens the stream and again once it closes,
   PERSISTENT while an open regular file is behind it. The TRANSIENT answer
   needs a character device, which is outside this program's own directory, so
   it is measured rather than filed here. */
say 'streamtype'
a = .Stream~new('f.txt')
say 'closed        [' || a~query('streamtype') || ']'
say 'open both     [' || a~open('both') || ']'
say 'open          [' || a~query('streamtype') || ']'
say 'close         [' || a~close || ']'
say 'closed        [' || a~query('streamtype') || ']'
/* The same answer through a read-only open, and through the implicit one a
   read performs. */
b = .Stream~new('f.txt')
say 'open read     [' || b~open('read') || ']'
say 'open          [' || b~query('streamtype') || ']'
say 'close         [' || b~close || ']'
c = .Stream~new('f.txt')
say 'implicit read [' || c~linein || ']'
say 'open          [' || c~query('streamtype') || ']'
say 'close         [' || c~close || ']'
/* A name nothing resolves to never opens, so it stays UNKNOWN. */
n = .Stream~new('nosuch.txt')
say 'missing       [' || n~query('streamtype') || ']'
say 'after a read  [' || n~linein || '] [' || n~query('streamtype') || ']'
