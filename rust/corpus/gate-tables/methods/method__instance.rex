/* Table C method rows: Method, instance arm. corpus/docs/class-set.txt
   records this class as `not-covered`, because
   no construction program is committed; a bare ~new raises 88.901 on the oracle.
   So ~new raises and no line below it is reached; the row's evidence
   is that raise, which is what the row set says there is to have.
   Derived by crates/rexx-exec/tests/gate_table_c.rs, which re-derives
   this file on every run and compares it in both directions. */
o = .Method~new
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
