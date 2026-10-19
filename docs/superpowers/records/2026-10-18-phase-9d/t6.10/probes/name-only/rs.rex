log=.array~new
.traceObject~option='P'
.traceObject~collector=log
r=.routine~new("testCallerStackFrame_ReplyStart", .array~of('   o=.test~new', '   o~count=2', '   o~m_reply', '   call syssleep 0.01', '   o~start("m_s")', '   o~m_wait', '', '   ::class test', '   ::method init', '     expose count', '     count=0', '', '   ::attribute count', '', '   ::method m_reply', '     expose count', '     reply', '     count-=1', '', '   ::method m_s', '     expose count', '     count-=1', '     call syssleep .01', '', '   ::method m_wait unguarded', '     expose count', '     guard on when count=0', '', '   ::options trace labels'))
r~call
.traceObject~collector=.nil
.traceObject~option='N'
n=0
do t over log
  s=t~makeString
  csf=''
  if pos(">I>",s)>0 then do
    csf=t~hasEntry("CALLERSTACKFRAME")
    if t~callerStackFrame<>.nil then do; csf=csf t~callerStackFrame~hasEntry("THREAD"); n+=t~callerStackFrame~hasEntry("THREAD"); end
  end
  say s '|' csf
end
say 'threads' n
