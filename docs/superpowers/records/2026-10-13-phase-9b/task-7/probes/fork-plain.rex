say 'before' 
pid = SysFork()
if pid = 0 then do
  say 'child: ppid matches' (SysQueryProcess('PPID') <> 0)
  exit 3
end
say 'parent: child pid positive' (pid > 0)
st = SysWait()
say 'parent: wait status' st
st = SysWait()
say 'parent: second wait (no child)' (st <> 768)
