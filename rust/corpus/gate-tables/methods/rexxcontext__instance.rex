/* Table C method rows: RexxContext, instance arm. corpus/docs/class-set.txt
   records this class as `not-covered`, because
   no construction program is committed; the reference says the user cannot create one and names a Rexx-level route instead -- utilityclasses.xml:7545 "They cannot be directly created by the user.".
   So ~new raises and no line below it is reached; the row's evidence
   is that raise, which is what the row set says there is to have.
   Derived by crates/rexx-exec/tests/gate_table_c.rs, which re-derives
   this file on every run and compares it in both directions. */
o = .RexxContext~new
say 'instance' o~hasMethod("args")
say 'instance' o~hasMethod("condition")
say 'instance' o~hasMethod("digits")
say 'instance' o~hasMethod("executable")
say 'instance' o~hasMethod("form")
say 'instance' o~hasMethod("fuzz")
say 'instance' o~hasMethod("interpreter")
say 'instance' o~hasMethod("invocation")
say 'instance' o~hasMethod("line")
say 'instance' o~hasMethod("name")
say 'instance' o~hasMethod("package")
say 'instance' o~hasMethod("rs")
say 'instance' o~hasMethod("stackFrames")
say 'instance' o~hasMethod("thread")
say 'instance' o~hasMethod("variables")
