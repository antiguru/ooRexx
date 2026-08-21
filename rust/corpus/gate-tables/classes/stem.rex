/* Table C wiring row: the .Stem environment entry, asked the questions
   the class surface is wired by -- ~id, ~class, ~superClass,
   ~superClasses, ~metaClass and ~isA(.Class). Derived from
   corpus/docs/class-set.txt by crates/rexx-exec/tests/gate_table_c.rs,
   which re-derives this file on every run and compares it in both
   directions. */
say 'id' .Stem~id
say 'class' .Stem~class
say 'superclass' .Stem~superClass
say 'superclasses' .Stem~superClasses~makeString('L', ' ')
say 'metaclass' .Stem~metaClass
say 'isa-class' .Stem~isA(.Class)
