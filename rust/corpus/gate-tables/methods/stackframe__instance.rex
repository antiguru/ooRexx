/* Table C method rows: StackFrame, instance arm. corpus/docs/class-set.txt
   records this class as `not-covered`, because
   no construction program is committed; the reference says the user cannot create one and names a Rexx-level route instead -- utilityclasses.xml:9407 "StackFrame instances cannot be directly created by the user.".
   So ~new raises and no line below it is reached; the row's evidence
   is that raise, which is what the row set says there is to have.
   Derived by crates/rexx-exec/tests/gate_table_c.rs, which re-derives
   this file on every run and compares it in both directions. */
o = .StackFrame~new
say 'instance' o~hasMethod("arguments")
say 'instance' o~hasMethod("context")
say 'instance' o~hasMethod("invocation")
say 'instance' o~hasMethod("line")
say 'instance' o~hasMethod("makeString")
say 'instance' o~hasMethod("name")
say 'instance' o~hasMethod("string")
say 'instance' o~hasMethod("target")
say 'instance' o~hasMethod("traceLine")
say 'instance' o~hasMethod("type")
