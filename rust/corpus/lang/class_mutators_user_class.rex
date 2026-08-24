/* Task 21: the four mutators on a class this file declares, where they must
   succeed. The readback is `~method`, which reads the class's own instance
   dictionary: `~hasMethod` sent to a class object asks its class behaviour
   and answers 0 either way, measured, so it cannot witness an instance-side
   define at all.

   `~define` stores the very object it was handed when that object has no
   scope yet, and `~defineMethods` never does -- the two identity rows below
   are what separates them, and the annotation row says the copy carries the
   directive's own annotations with it. */
m = .methods~z
.K~define("ZORK", m)
say (m~identityHash == .K~method("ZORK")~identityHash)
say .K~method("ZORK")~annotation("A")

/* The second argument omitted is a tombstone; `.nil` in its place takes the
   entry away. The names below are read back in
   class_mutator_define_nil_removes.rex, which needs its own file because
   `~method` on the removed one raises. */
.K~define("HIDDEN")
say .K~method("HIDDEN")

.K~delete("ZORK")
say .K~superClasses
.K~uninherit(.M)
say .K~superClasses

.K2~defineMethods(.methods)
say .K2~method("Z")
say (m~identityHash == .K2~method("Z")~identityHash)
say (.methods~z~identityHash == m~identityHash)
say .K2~method("Z")~annotation("A")

::method z
::annotate method z A 'aval'

::class M mixinclass Object
::class K inherit M
::class K2
