/* Table C wiring row: the .Package environment entry, asked the questions
   the class surface is wired by -- ~id, ~class, ~superClass,
   ~superClasses, ~metaClass and ~isA(.Class). Derived from
   corpus/docs/class-set.txt by crates/rexx-exec/tests/gate_table_c.rs,
   which re-derives this file on every run and compares it in both
   directions. */
say 'id' .Package~id
say 'class' .Package~class
say 'superclass' .Package~superClass
say 'superclasses' .Package~superClasses~makeString('L', ' ')
say 'metaclass' .Package~metaClass
say 'isa-class' .Package~isA(.Class)
