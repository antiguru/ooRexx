/* Table C method rows: Class, instance arm -- one line per method
   corpus/docs/class-methods.txt documents on this arm, asked of the
   instance `.Object~subclass('k')` answers, in the row set's own order. That
   expression is corpus/docs/class-set.txt's committed construction
   program for this class, and carrying one is what `covered` claims.
   Derived by crates/rexx-exec/tests/gate_table_c.rs, which re-derives
   this file on every run and compares it in both directions. */
o = .Object~subclass('k')
say 'instance' o~hasMethod("=")
say 'instance' o~hasMethod("==")
say 'instance' o~hasMethod("<>")
say 'instance' o~hasMethod("><")
say 'instance' o~hasMethod("\=")
say 'instance' o~hasMethod("\==")
say 'instance' o~hasMethod("activate")
say 'instance' o~hasMethod("annotation")
say 'instance' o~hasMethod("annotations")
say 'instance' o~hasMethod("baseClass")
say 'instance' o~hasMethod("defaultName")
say 'instance' o~hasMethod("define")
say 'instance' o~hasMethod("defineMethods")
say 'instance' o~hasMethod("delete")
say 'instance' o~hasMethod("enhanced")
say 'instance' o~hasMethod("id")
say 'instance' o~hasMethod("inherit")
say 'instance' o~hasMethod("isAbstract")
say 'instance' o~hasMethod("isMetaclass")
say 'instance' o~hasMethod("isSubclassOf")
say 'instance' o~hasMethod("metaClass")
say 'instance' o~hasMethod("method")
say 'instance' o~hasMethod("methods")
say 'instance' o~hasMethod("mixinClass")
say 'instance' o~hasMethod("package")
say 'instance' o~hasMethod("queryMixinClass")
say 'instance' o~hasMethod("subclass")
say 'instance' o~hasMethod("subclasses")
say 'instance' o~hasMethod("superClass")
say 'instance' o~hasMethod("superClasses")
say 'instance' o~hasMethod("uninherit")
