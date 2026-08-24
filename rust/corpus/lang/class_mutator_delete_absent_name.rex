/* Task 21: `~delete` of a name the class does not define is not an error --
   `deleteMethod` propagates only when the dictionary changed, and answers
   nothing either way. The `~method` below says the class is otherwise
   untouched. */
.K~delete("ZZZNOSUCH")
say .K~method("M")
say .K~superClasses

::class K
::method m
