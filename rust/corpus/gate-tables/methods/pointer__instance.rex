/* Table C method rows: Pointer, instance arm. corpus/docs/class-set.txt
   records this class as `unreachable`, because
   the reference says instances come only from native code -- utilityclasses.xml:6910 "can only be created using the native code application programming interfaces.".
   So ~new raises and no line below it is reached; the row's evidence
   is that raise, which is what the row set says there is to have.
   Derived by crates/rexx-exec/tests/gate_table_c.rs, which re-derives
   this file on every run and compares it in both directions. */
o = .Pointer~new
say 'instance' o~hasMethod("=")
say 'instance' o~hasMethod("==")
say 'instance' o~hasMethod("\=")
say 'instance' o~hasMethod("\==")
say 'instance' o~hasMethod("isNull")
