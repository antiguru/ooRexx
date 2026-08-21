/* Table C wiring row: the .Pointer environment entry, asked the questions
   the class surface is wired by -- ~id, ~class, ~superClass,
   ~superClasses, ~metaClass and ~isA(.Class). Derived from
   corpus/docs/class-set.txt by crates/rexx-exec/tests/gate_table_c.rs,
   which re-derives this file on every run and compares it in both
   directions. */
say 'id' .Pointer~id
say 'class' .Pointer~class
say 'superclass' .Pointer~superClass
say 'superclasses' .Pointer~superClasses~makeString('L', ' ')
say 'metaclass' .Pointer~metaClass
say 'isa-class' .Pointer~isA(.Class)
