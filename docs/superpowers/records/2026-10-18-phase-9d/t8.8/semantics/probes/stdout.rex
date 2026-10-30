call show 'initial'
call charout , 'abc'
call show 'buffered'
address system 'true'
call show 'handover'
exit
show:
  call lineout 'queries', arg(1) 'size=['.stdout~query('size')'] default=['.stdout~query('position')'] write=['.stdout~query('position write char')'] sys=['.stdout~query('position sys')']'
  call lineout 'queries'
  return
