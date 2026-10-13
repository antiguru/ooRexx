call show 'start'
call SysSleep 0.01
call show 'after sleep'
exit
show:
  call SysFileTree '/proc/self/task/*', t., 'DO'
  out = arg(1)':'
  do i = 1 to t.0
    out = out linein(t.i'/comm'); call stream t.i'/comm', 'c', 'close'
  end
  say out
  return
