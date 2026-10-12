/* Package~copy: a package over the same program whose routine, class and
   import tables, settings and annotations are its own copies. */
p = .context~package
q = p~copy
say (q == p) q~class~id q~name~right(9) (q~source == p~source) q~source~items
say p~routines~items q~routines~items (q~routines == p~routines)
say p~classes~items q~classes~items (q~classes == p~classes)
say p~publicRoutines~items q~publicRoutines~items p~publicClasses~items q~publicClasses~items
say p~definedMethods~items q~definedMethods~items (q~definedMethods == p~definedMethods)
say p~resources~items q~resources~items
say p~importedRoutines~items q~importedRoutines~items p~importedClasses~items q~importedClasses~items
say (q~local == p~local) q~local~items p~local~items
q~addRoutine('NEWR', .routine~new('n', 'return 5'))
say p~routines~items q~routines~items q~findRoutine('NEWR')~class~id p~findRoutine('NEWR')
q~addClass('NEWC', .object~subclass('nc'))
say p~classes~items q~classes~items q~findClass('NEWC')~id p~findClass('NEWC')
q~addPublicRoutine('PUBR', .routine~new('m', 'return 6'))
say p~publicRoutines~items q~publicRoutines~items
q~local~x = 1
say p~local~items q~local~items
say q~findClass('K')~id q~findRoutine('R')~call
say (q~findRoutine('R') == p~findRoutine('R'))
say q~annotations~items p~annotations~items (q~annotations == p~annotations)
say q~digits p~digits q~form q~fuzz q~trace
q~options('DIGITS', 20)
say q~digits p~digits
say (q~prolog == p~prolog) (q~prolog~class~id)
say q~sourceLine(1) '|' q~sourceSize
say q~namespaces~items p~namespaces~items
say (q~findClass('K') == .k) (q~findPublicClass('K') == .k)
say q~importedPackages~items p~importedPackages~items
q~addPackage(.package~new('x', 'say 1'))
say q~importedPackages~items p~importedPackages~items
say q~resource('rr')~items
say q~findNamespace('REXX')~name
say q~classes~items q~publicClasses~items
say q~definedMethods~items q~resources~items
r = .context~package~copy
say (r~local == p~local) (r~annotations == p~annotations) r~annotations~items r~annotation('n')
r~annotations['M'] = 1
say r~annotations~items p~annotations~items (r~definedMethods~items)
exit
::annotate package n 'v'
::class K public
::routine r public
  return 'r ran'
::method m
  return 1
::resource rr
line
::END
