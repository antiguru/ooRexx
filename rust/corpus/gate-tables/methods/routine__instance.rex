/* Table C method rows: Routine, instance arm. corpus/docs/class-set.txt
   records this class as `not-covered`, because
   no construction program is committed; a bare ~new raises 88.901 on the oracle.
   So ~new raises and no line below it is reached; the row's evidence
   is that raise, which is what the row set says there is to have.
   Derived by crates/rexx-exec/tests/gate_table_c.rs, which re-derives
   this file on every run and compares it in both directions. */
o = .Routine~new
say 'instance' o~hasMethod("[]")
say 'instance' o~hasMethod("annotation")
say 'instance' o~hasMethod("annotations")
say 'instance' o~hasMethod("call")
say 'instance' o~hasMethod("callWith")
say 'instance' o~hasMethod("package")
say 'instance' o~hasMethod("setSecurityManager")
say 'instance' o~hasMethod("source")
