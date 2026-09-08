/* Package's six find* readers and ~resource, each on a hit and on a miss.

   Two of them reach past this package's own tables and are what a body
   searching only ~classes gets wrong: ~findClass and ~findPublicClass both
   answer the Array class, which no table this program declares holds.
   ~findPublicRoutine is the C++'s own oddity -- it calls findRoutine, so a
   ::ROUTINE declared without PUBLIC answers through it. And ~findProgram
   answers a path String, not a Package. */

p = .context~package

say 'findClass own' p~findClass('OwnClass')~id
say 'findClass own lowercase' p~findClass('ownclass')~id
say 'findClass private' p~findClass('OwnClassPrivate')~id
say 'findClass imported' p~findClass('LIBKA')~id
say 'findClass environment' p~findClass('ARRAY')~id
say 'findClass miss' p~findClass('NoSuchClassHere')~class~id

say 'findPublicClass own' p~findPublicClass('OwnClass')~id
say 'findPublicClass private' p~findPublicClass('OwnClassPrivate')~class~id
say 'findPublicClass imported' p~findPublicClass('LIBKA')~id
say 'findPublicClass environment' p~findPublicClass('ARRAY')~id
say 'findPublicClass miss' p~findPublicClass('NoSuchClassHere')~class~id

say 'findRoutine own' p~findRoutine('OwnRoutine')~class~id
say 'findRoutine own lowercase' p~findRoutine('ownroutine')~class~id
say 'findRoutine private' p~findRoutine('OwnRoutinePrivate')~class~id
say 'findRoutine imported' p~findRoutine('LIBRA')~class~id
say 'findRoutine miss' p~findRoutine('NoSuchRoutine')~class~id

say 'findPublicRoutine own' p~findPublicRoutine('OwnRoutine')~class~id
say 'findPublicRoutine private is found too' p~findPublicRoutine('OwnRoutinePrivate')~class~id
say 'findPublicRoutine imported' p~findPublicRoutine('LIBRA')~class~id
say 'findPublicRoutine miss' p~findPublicRoutine('NoSuchRoutine')~class~id

/* The found Routine is the same object the table holds, which the tag says
   and the rendering does not. */
p~routines['OWNROUTINE']~objectName = 'TAGGED THROUGH THE TABLE'
say 'findRoutine answers that object' p~findRoutine('OwnRoutine')~objectName

say 'findNamespace hit' p~findNamespace('LIB')~class~id
say 'findNamespace lowercase' p~findNamespace('lib')~class~id
say 'findNamespace rexx' p~findNamespace('REXX')~class~id
say 'findNamespace rexx name' p~findNamespace('REXX')~name
say 'findNamespace miss' p~findNamespace('NoSuchNamespace')~class~id

/* The namespace answers the required package, and its own tables say so. */
lib = p~findNamespace('LIB')
say 'the namespace package holds' lib~classes['LIBKA']~id
say 'and its own routines' lib~routines['LIBRA']~class~id

found = p~findProgram('package_tables_lib.cls')
say 'findProgram class' found~class~id
say 'findProgram ends with the name' (right(found, 22) == 'package_tables_lib.cls')
say 'findProgram is absolute' (left(found, 1) == '/')
say 'findProgram miss' p~findProgram('no_such_file_at_all.rex')~class~id

say 'resource hit' p~resource('ONLYRESOURCE')[1]
say 'resource miss' p~resource('NOSUCH')~class~id

say 'findClass none:' try('p~findClass()')
say 'findRoutine none:' try('p~findRoutine()')
say 'findNamespace none:' try('p~findNamespace()')
say 'findProgram none:' try('p~findProgram()')
say 'resource none:' try('p~resource()')

exit 0

try:
  parse arg trysource
  signal on syntax name raised
  interpret 'tryanswer =' trysource
  return 'answered' tryanswer
raised:
  return 'raised' rc || '.' || condition('e')

::requires 'package_tables_lib.cls' namespace LIB

::routine OwnRoutine public
  return 1

::routine OwnRoutinePrivate
  return 2

::class OwnClass public

::class OwnClassPrivate

::resource ONLYRESOURCE
the only line
::END
