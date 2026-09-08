/* Package's ten table readers, on a package carrying more than one of every
   kind: two classes public and one not, two routines public and one not, two
   unattached methods, two resources, and one ::REQUIRES under a namespace.
   A body answering an empty table or a one-entry table fails here.

   Every entry is read back BY NAME and its own class asked, because the
   table's rendering is `a StringTable` whatever is inside it.

   This program's stdout is compared as a sorted multiset -- the tables are
   hash-ordered on both sides and neither order reproduces the other.

   Every name a table is keyed under here is at most seven bytes, so that the
   index string a DO OVER binds sits in the handle rather than on the heap.
   That is a workaround for a defect this program found and does not own:
   under collect-on-every-allocation a DO OVER over one of the interpreter's
   own StringTables binds a dead handle once the index is long enough to be
   allocated, which `docs/superpowers/records/2026-09-07-phase-5i-introspection/
   found-defect-do-over-native-table-rooting.md` reproduces with `.methods`
   alone. */

p = .context~package
call dump 'classes', p~classes
call dump 'publicClasses', p~publicClasses
call dump 'importedClasses', p~importedClasses
call dump 'routines', p~routines
call dump 'publicRoutines', p~publicRoutines
call dump 'importedRoutines', p~importedRoutines
call dump 'definedMethods', p~definedMethods
call dump 'resources', p~resources
call dump 'namespaces', p~namespaces

ip = p~importedPackages
say 'importedPackages class' ip~class~id 'items' ip~items
do i = 1 to ip~items
  say 'importedPackages['i'] class' ip[i]~class~id
end

/* Each table is a copy: a write into one is invisible to the next ask. */
one = p~resources
one['ZZZ'] = 'written into the copy'
say 'a write into the copy is not in the package' (p~resources['ZZZ'] == .nil)

/* The resource bodies are Arrays of lines, in file order -- an order the
   oracle specifies, so these lines are read by position. */
r2 = p~resource('SB')
say 'resource class' r2~class~id 'items' r2~items
say 'resource line 1 [' || r2[1] || ']'
say 'resource line 2 [' || r2[2] || ']'
say 'resource lookup upcases' p~resource('sb')~items
say 'resource miss' p~resource('NOSUCH')~class~id

exit 0

dump: procedure
  use arg label, table
  count = 0
  do name over table
    count = count + 1
    say label '['name'] is a' table[name]~class~id
  end
  say label 'class' table~class~id 'entries' count
  return

::requires 'package_tables_lib.cls' namespace LIB

::method MA
  return 1

::method MB
  return 2

::routine RA public
  return 1

::routine RB public
  return 2

::routine RC
  return 3

::class KA public

::class KB public

::class KC

::resource SA
only line
::END

::resource SB
first line
second line
::END
