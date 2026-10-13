say 'before'
t = .worker~new
t~start('spin')
call SysSleep 0.2
pid = SysFork()
if pid = 0 then do
  say 'child: alive'
  exit 3
end
say 'parent: child pid positive' (pid > 0)
st = SysWait()
say 'parent: wait status' st
t~stop = 1
exit
::class worker
::attribute stop
::method init
  expose stop; stop = 0
::method spin
  expose stop
  n = 0
  do until stop
    n = n + 1
    if n // 1000 = 0 then call SysSleep 0.001
  end
