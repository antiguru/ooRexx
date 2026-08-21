/* Table C method rows: VariableReference, instance arm. corpus/docs/class-set.txt
   records this class as `not-covered`, because
   no construction program is committed; the reference says the user cannot create one and names a Rexx-level route instead -- utilityclasses.xml:12556-12557 "Calling the new method to create a VariableReference instance is not allowed.".
   So ~new raises and no line below it is reached; the row's evidence
   is that raise, which is what the row set says there is to have.
   Derived by crates/rexx-exec/tests/gate_table_c.rs, which re-derives
   this file on every run and compares it in both directions. */
o = .VariableReference~new
say 'instance' o~hasMethod("name")
say 'instance' o~hasMethod("request")
say 'instance' o~hasMethod("unknown")
say 'instance' o~hasMethod("value")
