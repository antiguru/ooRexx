/* A copied package is a package of its own: what ~addRoutine, ~addClass,
   ~addPublicClass, ~addPackage and ~setSecurityManager write on the copy
   the original does not see, and the reverse. */
p = .context~package
p2 = p~copy
say 'copy' p2~class~id (p2 == p) (p2~name == p~name)
r = .routine~new('r', 'return "added"')
p2~addRoutine('ADDEDR', r)
say 'routine' (p~findRoutine('ADDEDR') == .nil) (p2~findRoutine('ADDEDR') == r)
say 'routines' p~routines~hasIndex('ADDEDR') p2~routines~hasIndex('ADDEDR')
p2~addClass('ADDEDC', .array)
say 'class' p~classes~hasIndex('ADDEDC') p2~classes~hasIndex('ADDEDC')
say 'findclass' (p~findClass('ADDEDC') == .nil) (p2~findClass('ADDEDC') == .array)
p2~addPublicClass('PUBC', .list)
say 'public' p~publicClasses~hasIndex('PUBC') p2~publicClasses~hasIndex('PUBC')
q = .package~new('q', .array~of('::class qk public'))
p2~addPackage(q)
say 'imports' p~importedPackages~items p2~importedPackages~items (p2~importedPackages[1] == q)
say 'imported' p~importedClasses~hasIndex('QK') p2~importedClasses~hasIndex('QK')
say 'manager' p2~setSecurityManager(.sm~new)
x = .local
p~addRoutine('ORIGR', r)
say 'reverse' p~routines~hasIndex('ORIGR') p2~routines~hasIndex('ORIGR')
say 'mine' p~routines~hasIndex('MINE') p2~routines~hasIndex('MINE')
/* A copy of a copy, and of a Package subclass instance. */
p3 = p2~copy
say 'copy2' p3~routines~hasIndex('ADDEDR') p3~classes~hasIndex('ADDEDC')
s = .mypkg~new('s', .array~of('::routine sr public', 'return 1'))
s2 = s~copy
s2~addRoutine('S2R', r)
say 'subclass' s2~class~id s2~name s~routines~hasIndex('S2R') s2~routines~hasIndex('S2R') s2~routines~hasIndex('SR')
/* The REXX package copies like any other. Only the REXX package itself
   refuses an addition: the copy is a package of its own. */
x = .array~package
c = x~copy
say 'rexx' c~name (c == x) c~class~id (c~findClass('ARRAY') == .array) (c~classes~items > 0)
c~addClass('XADDED', .list)
c~addRoutine('XR', r)
say 'rexx copy' c~classes~hasIndex('XADDED') x~classes~hasIndex('XADDED') (c~findClass('XADDED') == .list) c~routines~hasIndex('XR') x~routines~hasIndex('XR')
signal on syntax name rexxadd
x~addClass('XADDED', .list)
say 'rexx addClass answered'
exit
rexxadd:
say 'rexx addClass' condition('O')~code
exit
::routine mine
  return 1
::class sm
::method local
  say 'manager checked'
  return 0
::method unknown
  return 0
::class mypkg subclass package
