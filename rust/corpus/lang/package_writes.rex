/* Package's four writes -- ~addRoutine, ~addPublicRoutine, ~addPackage and
   ~loadPackage -- each witnessed by the read that should see it, with the
   BEFORE value printed as well as the after: a table that was already
   non-empty makes "the write worked" indistinguishable from "the read
   answers something".

   All three add* answer the RECEIVING package and not the thing added, which
   is what the object-name tags say. ~loadPackage answers the loaded package
   instead, and ~addPackage is what fills ~importedClasses and
   ~importedRoutines, so those two rows cannot be witnessed without it.

   This program's stdout is compared as a sorted multiset -- the tables it
   prints are hash-ordered on both sides and neither order reproduces the
   other. Its names are short for package_tables.rex's reason.

   No file in this repository is instantiated through .Package~new: the file
   form here is ~loadPackage of a .cls this program's own ::REQUIRES has
   already loaded, and the in-memory form is an array of lines. */

p = .context~package
p~objectName = 'THE RECEIVING PACKAGE'

call dump 'before routines', p~routines
call dump 'before publicRoutines', p~publicRoutines
call dump 'before importedClasses', p~importedClasses
call dump 'before importedRoutines', p~importedRoutines
say 'before importedPackages' p~importedPackages~items

added = .Routine~new('ADDED', .array~of('return 42'))
added~objectName = 'THE ADDED ROUTINE'

back = p~addRoutine('ADDA', added)
say 'addRoutine answers' back~class~id back~objectName
call dump 'after addRoutine', p~routines
say 'the table holds the object added' p~routines['ADDA']~objectName
say 'and so does findRoutine' p~findRoutine('ADDA')~objectName
say 'and it runs' p~findRoutine('ADDA')~call

back = p~addPublicRoutine('ADDB', added)
say 'addPublicRoutine answers' back~class~id back~objectName
call dump 'after addPublicRoutine publicRoutines', p~publicRoutines
call dump 'after addPublicRoutine routines', p~routines

/* .ROUTINES is the package's own table, so an addition is visible there. */
call dump 'dot ROUTINES', .routines

dep = p~loadPackage('package_tables_dep.cls')
say 'loadPackage answers a' dep~class~id
say 'loadPackage answers the loaded package, not the receiver' ,
    (dep~objectName == 'THE RECEIVING PACKAGE')
depname = dep~name
say 'loadPackage name ends with the file' (right(depname, 22) == 'package_tables_dep.cls')
say 'after loadPackage importedPackages' p~importedPackages~items
call dump 'after loadPackage importedClasses', p~importedClasses
call dump 'after loadPackage importedRoutines', p~importedRoutines

/* Adding the same package a second time does not grow the list. */
back = p~addPackage(dep)
say 'addPackage answers' back~class~id back~objectName
say 'adding one already imported leaves the count' p~importedPackages~items

back = p~addPackage(dep, 'NSD')
call dump 'after addPackage namespaces', p~namespaces
depns = p~findNamespace('NSD')~name
say 'findNamespace answers the added package' (right(depns, 22) == 'package_tables_dep.cls')

/* The interpreter's own package can be imported, which the arity table found
   and a body that takes only a program's package refuses: it contributes its
   62 public classes and no routines. */
rexxclasses = 0
do zz over p~importedClasses
  rexxclasses = rexxclasses + 1
  if zz == '' then nop
end
say 'importedClasses before importing REXX' rexxclasses
back = p~addPackage(.Class~package, 'NSR')
say 'addPackage of the REXX package answers' back~class~id back~objectName
say 'importedPackages now' p~importedPackages~items
rexxclasses = 0
do zz over p~importedClasses
  rexxclasses = rexxclasses + 1
  if zz == '' then nop
end
say 'importedClasses after importing REXX' rexxclasses
rexxroutines = 0
do zz over p~importedRoutines
  rexxroutines = rexxroutines + 1
  if zz == '' then nop
end
say 'importedRoutines after importing REXX' rexxroutines
say 'the namespace it was added under names' p~findNamespace('NSR')~name

say 'addRoutine one argument:' try("p~addRoutine('X')")
say 'addRoutine not a routine:' try("p~addRoutine('X', 5)")
say 'addPublicRoutine not a routine:' try("p~addPublicRoutine('X', 5)")
say 'addPackage not a package:' try('p~addPackage(5)')
say 'addPackage none:' try('p~addPackage()')
say 'loadPackage no such file:' try("p~loadPackage('no_such_file_at_all.rex')")

exit 0

dump:
  parse arg dumplabel
  dumptable = arg(2)
  dumpcount = 0
  do dumpname over dumptable
    dumpcount = dumpcount + 1
    say dumplabel '['dumpname'] is a' dumptable[dumpname]~class~id
  end
  say dumplabel 'entries' dumpcount
  return

try:
  parse arg trysource
  signal on syntax name raised
  interpret 'tryanswer =' trysource
  return 'answered' tryanswer
raised:
  return 'raised' rc || '.' || condition('e')

::routine RD public
  return 1

::routine RC
  return 2
