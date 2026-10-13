parse arg file
call SysMkDir 'n'
do name over 'a', 'b', 'c', 'A', 'z', 'Z', '-', ']', '^', '!', '[', '=', ':', '\', 'x.y', 'ab', 'a]', 'c]', '[a', 'A]'
  call charout 'n/'name, ''
  call charout 'n/'name
end
call directory 'n'
do while lines(file) > 0
  p = linein(file)
  do o over 'o', 'oi'
    drop f.
    call SysFileTree p, f., o
    a = .array~new
    do j = 1 to f.0; a~append(filespec('n', f.j)); end
    a~sort
    say p '|' o '|' a~toString('l', ' ')
  end
end
call directory '..'
'rm -rf n'
