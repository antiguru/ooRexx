/* Table C wiring row: the .Queue environment entry, asked the questions
   the class surface is wired by -- ~id, ~class, ~superClass,
   ~superClasses, ~metaClass and ~isA(.Class). Derived from
   corpus/docs/class-set.txt by crates/rexx-exec/tests/gate_table_c.rs,
   which re-derives this file on every run and compares it in both
   directions. */
say 'id' .Queue~id
say 'class' .Queue~class
say 'superclass' .Queue~superClass
say 'superclasses' .Queue~superClasses~makeString('L', ' ')
say 'metaclass' .Queue~metaClass
say 'isa-class' .Queue~isA(.Class)
