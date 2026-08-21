/* Table C wiring row: the .RexxInfo environment entry, asked the questions
   the class surface is wired by -- ~id, ~class, ~superClass,
   ~superClasses, ~metaClass and ~isA(.Class). Derived from
   corpus/docs/class-set.txt by crates/rexx-exec/tests/gate_table_c.rs,
   which re-derives this file on every run and compares it in both
   directions. */
say 'id' .RexxInfo~id
say 'class' .RexxInfo~class
say 'superclass' .RexxInfo~superClass
say 'superclasses' .RexxInfo~superClasses~makeString('L', ' ')
say 'metaclass' .RexxInfo~metaClass
say 'isa-class' .RexxInfo~isA(.Class)
