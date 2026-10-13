p = .context~package
r = p~findRoutine('SysSleep')
say 'findRoutine nil' (r == .nil)
if r \== .nil then do
  say 'string' r~string
  say 'objectname' r~objectName
  say 'pkg nil' (r~package == .nil)
  if r~package \== .nil then say 'pkg name' r~package~name
  say 'source items' r~source~items
  say 'call' r~call(0)
  say 'isGuarded' r~isGuarded
end
say 'same twice' (p~findRoutine('SysSleep') == r)
say 'lower' (p~findRoutine('syssleep') == r)
say 'publicRoutines' p~publicRoutines~items
say 'routines' p~routines~items
say 'loadLibrary' p~loadLibrary('rexxutil')
say 'loadLibrary upper' p~loadLibrary('REXXUTIL')
say 'loadLibrary REXX' p~loadLibrary('REXX')
call SysSleep 0
say 'call' result
say SysVersion()~word(1)
::requires 'rexxutil' library
