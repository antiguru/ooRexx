say 'go'
call SysSleep 0
say 'rc' result
p = .context~package
say 'imported' p~importedPackages~items
do pk over p~importedPackages
  say 'pkg name' pk~name
  say 'pkg routines' pk~routines~items
  say 'pkg publicRoutines' pk~publicRoutines~items
  say 'pkg classes' pk~classes~items
  say 'pkg sourcesize' pk~sourceSize
end
say 'findRoutine SysSleep' (p~findRoutine('SysSleep') == .nil)
say 'findRoutine SYSSLEEP' (p~findRoutine('SYSSLEEP') == .nil)
say 'routines' p~routines~items
say 'libraries' p~libraries~items
say 'done'
::requires 'rexxutil' library
