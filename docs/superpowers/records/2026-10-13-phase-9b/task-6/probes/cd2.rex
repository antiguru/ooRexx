current = directory()
call SysMkDir 'ab'
call SysMkDir 'a b'
call value 'HOME', current, 'ENVIRONMENT'
call t 'cd ~'
call t 'cd ~/ab'
call t 'cd ~root'
call t 'cd ~root/'
call t 'cd ~nosuchuser.x'
call t 'cd "a"b'
call t 'cd a\ b'
call t 'cd    ab'
call t 'cd ab/x/..'
call t 'cd'
call SysRmDir 'ab'
call SysRmDir 'a b'
exit
t: parse arg c
  call directory current
  address system c
  say left(c, 20) rc directory()
  return
