/* While interactive tracing is on, a TRACE instruction leaves the setting
   alone, a bare TRACE and an invalid setting included; the TRACE builtin
   still changes it. Package~trace answers a debug setting with its '?'. */
say trace()
trace ?c
say trace()
trace
say trace()
trace off
say trace()
trace ("*invalid*")
say trace()
trace value 'R'
say trace()
trace ?
say trace()
call trace 'N'
say trace()
trace ?e
say trace()
x = trace('O')
say x trace()
p = .package~new('pkc', .array~of('return 1', '::options trace ?c'))
say p~trace p~options('TRACE')
p = .package~new('pke', .array~of('return 1', '::options trace ?e'))
say p~trace p~options('TRACE')
p = .package~new('pkn', .array~of('return 1', '::options trace c'))
say p~trace p~options('TRACE')
