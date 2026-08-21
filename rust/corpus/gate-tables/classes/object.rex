/* Table C wiring row: the .Object environment entry, asked the questions
   the class surface is wired by -- ~id, ~class, ~superClass,
   ~superClasses, ~metaClass and ~isA(.Class). Derived from
   corpus/docs/class-set.txt by crates/rexx-exec/tests/gate_table_c.rs,
   which re-derives this file on every run and compares it in both
   directions. */
say 'id' .Object~id
say 'class' .Object~class
say 'superclass' .Object~superClass
say 'superclasses' .Object~superClasses~makeString('L', ' ')
say 'metaclass' .Object~metaClass
say 'isa-class' .Object~isA(.Class)
