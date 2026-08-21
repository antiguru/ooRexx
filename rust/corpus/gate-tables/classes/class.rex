/* Table C wiring row: the .Class environment entry, asked the questions
   the class surface is wired by -- ~id, ~class, ~superClass,
   ~superClasses, ~metaClass and ~isA(.Class). Derived from
   corpus/docs/class-set.txt by crates/rexx-exec/tests/gate_table_c.rs,
   which re-derives this file on every run and compares it in both
   directions. */
say 'id' .Class~id
say 'class' .Class~class
say 'superclass' .Class~superClass
say 'superclasses' .Class~superClasses~makeString('L', ' ')
say 'metaclass' .Class~metaClass
say 'isa-class' .Class~isA(.Class)
