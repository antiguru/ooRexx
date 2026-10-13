p = .context~package
r = p~findRoutine('SysSleep')
say 'findRoutine nil' (r == .nil)
if r \== .nil then do
  say 'class' r~class~id
  say 'name' r~name
  say 'pkg nil' (r~package == .nil)
  if r~package \== .nil then say 'pkg name' r~package~name
  say 'source items' r~source~items
  say 'call' r~call(0)
end
r2 = p~findRoutine('SysVersion')
say 'sysversion nil' (r2 == .nil)
say 'same twice' (p~findRoutine('SysSleep') == r)
say 'publicRoutines' p~publicRoutines~items
say 'loadLibrary' .Package~loadLibrary('rexxutil')
say 'loadLibrary upper' .Package~loadLibrary('REXXUTIL')
::requires 'rexxutil' library
