m3 = .message~new(.t~new, 'boom')
m4 = .message~new(m3, 'send')
m4~start
m4~wait
call SysSleep 0.3
say m3~hasError m4~hasError (m3~errorCondition == m4~errorCondition) (m3~errorCondition == .nil) (m4~errorCondition == .nil)
::class t
::method boom
  return 1/0
