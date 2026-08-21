/* Table C wiring row: the .Ticker environment entry, asked the questions
   the class surface is wired by -- ~id, ~class, ~superClass,
   ~superClasses, ~metaClass and ~isA(.Class). Derived from
   corpus/docs/class-set.txt by crates/rexx-exec/tests/gate_table_c.rs,
   which re-derives this file on every run and compares it in both
   directions. */
say 'id' .Ticker~id
say 'class' .Ticker~class
say 'superclass' .Ticker~superClass
say 'superclasses' .Ticker~superClasses~makeString('L', ' ')
say 'metaclass' .Ticker~metaClass
say 'isa-class' .Ticker~isA(.Class)
