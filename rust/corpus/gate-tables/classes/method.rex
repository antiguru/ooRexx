/* Table C wiring row: the .Method environment entry, asked the questions
   the class surface is wired by -- ~id, ~class, ~superClass,
   ~superClasses, ~metaClass and ~isA(.Class). Derived from
   corpus/docs/class-set.txt by crates/rexx-exec/tests/gate_table_c.rs,
   which re-derives this file on every run and compares it in both
   directions. */
say 'id' .Method~id
say 'class' .Method~class
say 'superclass' .Method~superClass
say 'superclasses' .Method~superClasses~makeString('L', ' ')
say 'metaclass' .Method~metaClass
say 'isa-class' .Method~isA(.Class)
