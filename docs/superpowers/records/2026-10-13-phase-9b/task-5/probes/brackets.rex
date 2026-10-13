call directory 'n'
pats = .array~of('[[=a=]]', '[[.a.]]', '[[.-.]]', '[a-[.c.]]', '[[=a=]b]', '[[.a.]-c]', '[[=a=]-c]', '[a-[=c=]]', '[[.ab.]]', '[[=ab=]]', '[[.].]]', '[[.[.]]', '[[=.=]]', '[[..]]', '[[==]]', '[[.a]', '[[=a]', '[[.a.]', '[![.a.]]', '[[.A.]]', '[[=A=]]')
do p over pats
  do o over 'o', 'oi'
    drop f.
    call SysFileTree p, f., o
    a = .array~new
    do i = 1 to f.0; a~append(filespec('n', f.i)); end
    a~sort
    say left(p, 12) left(o, 3) a~toString('l', ' ')
  end
end
