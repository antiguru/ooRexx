/* Task 21: `~define(name, .nil)` is not the tombstone the omitted argument
   leaves. `class_mutators_user_class.rex` reads a tombstone back as `The NIL
   object`; this reads the `.nil` form back and it is gone. */
.K~define("ZORK", .methods~z)
.K~define("ZORK", .nil)
say .K~method("ZORK")

::method z
::class K
