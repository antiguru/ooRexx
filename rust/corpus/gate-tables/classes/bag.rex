/* Table C wiring row: the .Bag environment entry, asked the questions
   the class surface is wired by -- ~id, ~class, ~superClass,
   ~superClasses, ~metaClass and ~isA(.Class). Derived from
   corpus/docs/class-set.txt by crates/rexx-exec/tests/gate_table_c.rs,
   which re-derives this file on every run and compares it in both
   directions. */
say 'id' .Bag~id
say 'class' .Bag~class
say 'superclass' .Bag~superClass
say 'superclasses' .Bag~superClasses~makeString('L', ' ')
say 'metaclass' .Bag~metaClass
say 'isa-class' .Bag~isA(.Class)
