p = .stream~method('INIT')~package
say p~name
say p~source~items
say p~sourceline(1)'|'
say p~sourcesize
q = .array~method('SORT')~package
say q~name q~source~items q~sourcesize
say .datetime~method("INIT")~package~name (.alarm~method("INIT")~package == .array~method("SORT")~package)
say .orderedcollection~method('SORTWITH')~package~name
say .context~package~findRoutine('SYSVERSION')~package~name
