f = 'dub.txt'
call lineout f, 'x'
call lineout f
call z '2020-03-29', '01:30:00'
call z '2020-03-29', '00:59:59'
call z '2020-03-29', '02:00:00'
call z '2020-10-25', '01:30:00'
call z '2021-03-28', '01:30:00'
call z '2045-03-26', '01:30:00'
exit
z: procedure expose f
  r = SysSetFileDateTime(f, arg(1), arg(2))
  'stat -c %Y' f '> s.out'
  s = linein('s.out'); call stream 's.out', 'c', 'close'
  say arg(1) arg(2) r s SysGetFileDateTime(f)
  return
