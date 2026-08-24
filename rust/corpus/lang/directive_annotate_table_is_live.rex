/* ~annotations answers a table the program can add to, and ~annotation reads
   the addition back. RexxClass::getAnnotations and
   BaseExecutable::getAnnotations create the table on the first ask and store
   it in the object's own field (classes/ClassClass.cpp:325,
   execution/BaseExecutable.cpp:378), so it is one live table per annotated
   thing and not a snapshot of what the directives said.

   The class rows and the method rows are separate because the two reach
   different C++ bodies and, here, different storage. The method rows are the
   sharper of the two: ~method retrieves the one Method object the class's
   dictionary holds (RexxClass::method, classes/ClassClass.cpp:984), so an
   addition made through one ~method send is visible through the next one --
   which a build handing out a fresh table per send answers as .nil.

   The last row is the control that keeps the additions from passing for the
   wrong reason: a second method of the same class has its own table and must
   not see what was added to M's. */
.K~annotations~put("added late", "EXTRA")
say .K~annotation("EXTRA")
say .K~annotation("AUTHOR")

say .K~method("M")~annotation("EXTRA")
.K~method("M")~annotations~put("added late too", "EXTRA")
say .K~method("M")~annotation("EXTRA")
say .K~method("P")~annotation("EXTRA")

::class K
::annotate class K author "from the class"
::method m
  return 1
::method p
  return 2
