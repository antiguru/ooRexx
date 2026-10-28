f = .context~stackFrames
do x over f; say 'frame' x~type filespec('name', x~name) x~line; end
pk = f[f~items]~executable~package
say 'pkg' filespec('name', pk~name)
pk~setSecurityManager(.mgr~new)
say 'set'
::class mgr
::method unknown; use arg n, a; say 'MGR' n filespec('name', a[1]~name); return 0
::method command; use arg info; say 'MGR command' info~command; info~rc=42; return 1
