/* A helper for package_tables.rex and package_writes.rex: two public classes
   and two public routines, so that a body answering a one-entry table cannot
   pass. Directives only -- it has no leading code section, so nothing runs
   when it is required, loaded or named.

   Every name here is at most seven bytes, for the reason package_tables.rex
   gives. */

::routine LIBRA public
  return 1

::routine LIBRB public
  return 2

::routine LIBRC
  return 3

::class LIBKA public

::class LIBKB public

::class LIBKC
