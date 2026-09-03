/* Table C method rows: Method, instance arm -- one line per method
   corpus/docs/class-methods.txt documents on this arm, asked of the
   instance `.Object~method('objectName')` answers, in the row set's own order. That
   expression is corpus/docs/class-set.txt's committed construction
   program for this class, and carrying one is what `covered` claims.
   Derived by crates/rexx-exec/tests/gate_table_c.rs, which re-derives
   this file on every run and compares it in both directions. */
o = .Object~method('objectName')
say 'instance' o~hasMethod("annotation")
say 'instance' o~hasMethod("annotations")
say 'instance' o~hasMethod("isAbstract")
say 'instance' o~hasMethod("isAttribute")
say 'instance' o~hasMethod("isConstant")
say 'instance' o~hasMethod("isGuarded")
say 'instance' o~hasMethod("isPackage")
say 'instance' o~hasMethod("isPrivate")
say 'instance' o~hasMethod("isProtected")
say 'instance' o~hasMethod("package")
say 'instance' o~hasMethod("scope")
say 'instance' o~hasMethod("setGuarded")
say 'instance' o~hasMethod("setPrivate")
say 'instance' o~hasMethod("setProtected")
say 'instance' o~hasMethod("setSecurityManager")
say 'instance' o~hasMethod("setUnguarded")
say 'instance' o~hasMethod("source")
