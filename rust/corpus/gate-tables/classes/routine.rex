/* Table C wiring row: the .Routine environment entry, asked the questions
   the class surface is wired by -- ~id, ~class, ~superClass,
   ~superClasses, ~metaClass and ~isA(.Class). Derived from
   corpus/docs/class-set.txt by crates/rexx-exec/tests/gate_table_c.rs,
   which re-derives this file on every run and compares it in both
   directions. */
say 'id' .Routine~id
say 'class' .Routine~class
say 'superclass' .Routine~superClass
say 'superclasses' .Routine~superClasses~makeString('L', ' ')
say 'metaclass' .Routine~metaClass
say 'isa-class' .Routine~isA(.Class)
