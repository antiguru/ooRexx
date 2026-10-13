call directory 'n'
do c over .array~of('[^a]x|o', '[]x]|o', 'a[|o', '[[:digit:]]*|o', '[[:nosuch:]a]|o', '[_-a]|oi', '[!a-z]|o', '[[:upper:]]|oi')
  parse var c spec '|' opts
  drop f.
  call SysFileTree spec, f., opts
  a = .array~new
  do i = 1 to f.0; a~append(filespec('n', f.i)); end
  a~sort
  say left(c, 16) a~toString('l', ' ')
end
