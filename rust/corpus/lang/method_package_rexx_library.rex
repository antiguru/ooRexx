/* A method of the interpreter's own library is in the REXX package, whose
   source has no lines. */
p = .stream~method('INIT')~package
say p~name p~source~items p~sourceline(1)'|' p~sourcesize
say (p == .array~method('SORT')~package) (p == .alarm~method('INIT')~package)
say .datetime~method('INIT')~package~name .orderedcollection~method('SORTWITH')~package~name
say (.context~package~findRoutine('LOCAL')~package == .context~package)
::routine local
