p = .context~package
ir = p~importedRoutines
say 'imported' ir~class~id ir~items
say 'has SYSSLEEP' ir~hasIndex('SYSSLEEP') 'SysSleep' ir~hasIndex('SysSleep')
say 'has SYSADDREXXMACRO' ir~hasIndex('SYSADDREXXMACRO')
say 'has SYSFORK' ir~hasIndex('SYSFORK')
say 'has FILESPEC' ir~hasIndex('FILESPEC')
r = p~findRoutine('SysSleep')
say 'same as imported' (ir['SYSSLEEP'] == r)
say 'findPublicRoutine' (p~findPublicRoutine('SysSleep') == .nil)
say 'routines' p~routines~items 'public' p~publicRoutines~items
say 'pkgname' r~package~name 'same rexx pkg' (r~package == .context~package~findNamespace('REXX'))
say 'two calls ir same' (p~importedRoutines == ir)
::requires 'rexxutil' library
