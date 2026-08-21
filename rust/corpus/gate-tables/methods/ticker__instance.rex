/* Table C method rows: Ticker, instance arm. corpus/docs/class-set.txt
   records this class as `not-covered`, because
   no construction program is committed; a bare ~new raises 93.901 on the oracle.
   So ~new raises and no line below it is reached; the row's evidence
   is that raise, which is what the row set says there is to have.
   Derived by crates/rexx-exec/tests/gate_table_c.rs, which re-derives
   this file on every run and compares it in both directions. */
o = .Ticker~new
say 'instance' o~hasMethod("attachment")
say 'instance' o~hasMethod("cancel")
say 'instance' o~hasMethod("canceled")
say 'instance' o~hasMethod("cancelled")
say 'instance' o~hasMethod("init")
say 'instance' o~hasMethod("interval")
