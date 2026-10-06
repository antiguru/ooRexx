m = .c~new~start('spin')
do i = 1 to 6
  t.i = SysGettid()
  call SysSleep 0.05
end
same = 1
do i = 2 to 6
  if t.i \= t.1 then same = 0
end
say 'main same tid' same
.c~stop = 1
m~wait
::requires 'rxunixsys' LIBRARY
::class c
::attribute stop class
::method spin
  do i = 1 to 50
    x = SysGettid()
    call SysSleep 0.01
    if .c~stop == 1 then leave
  end
  return 0
