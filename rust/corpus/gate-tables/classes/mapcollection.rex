/* Table C wiring row: the .MapCollection environment entry, asked the questions
   the class surface is wired by -- ~id, ~class, ~superClass,
   ~superClasses, ~metaClass and ~isA(.Class). Derived from
   corpus/docs/class-set.txt by crates/rexx-exec/tests/gate_table_c.rs,
   which re-derives this file on every run and compares it in both
   directions. */
say 'id' .MapCollection~id
say 'class' .MapCollection~class
say 'superclass' .MapCollection~superClass
say 'superclasses' .MapCollection~superClasses~makeString('L', ' ')
say 'metaclass' .MapCollection~metaClass
say 'isa-class' .MapCollection~isA(.Class)
